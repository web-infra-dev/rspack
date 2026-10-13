import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { setImmediate } from 'node:timers/promises';
import { lazyCompilationMiddleware } from '@rspack/core';

function session(compiler) {
  const results = [];
  const waiters = [];
  const watching = compiler.watch({}, (error, stats) => {
    const result = { error, stats };
    if (waiters.length) waiters.shift()(result);
    else results.push(result);
  });
  return {
    watching,
    async next() {
      const { error, stats } = results.length ? results.shift() : await new Promise(resolve => waiters.push(resolve));
      if (error) throw error;
      return stats;
    },
    close() { return new Promise(resolve => watching.close(resolve)); },
  };
}

const options = () => ({ mode: 'development', devtool: false, entry: './index.js' });

export default [
  {
    description: 'records every manual invalidation synchronously and preserves source payloads and watch lifetimes',
    options,
    async build(context, compiler) {
      const events = [];
      let returned = false;
      compiler.hooks.watchInvalidation.tap('Origins', event => {
        assert.equal(returned, false);
        events.push(event);
      });
      const first = session(compiler);
      let initial;
      try {
        initial = (await first.next()).compilation;
        const watchId = initial.rebuildOrigin.watchId;
        assert.deepEqual(initial.rebuildOrigin, {
          watchId, throughRevision: 0,
          causes: [{ watchId, revision: 0, cause: { kind: 'initial' } }], consumedLazyKeys: [],
        });
        first.watching.suspend();
        first.watching.invalidate();
        first.watching.invalidate();
        const changed = new Set(['changed.js']);
        const removed = new Set(['removed.js']);
        first.watching.invalidateWithChangesAndRemovals(changed, removed);
        changed.add('injected');
        removed.clear();
        returned = true;
        assert.deepEqual(events.map(event => [event.revision, event.cause]), [
          [1, { kind: 'unknown' }], [2, { kind: 'unknown' }],
          [3, { kind: 'source', changed: ['changed.js'], removed: ['removed.js'] }],
        ]);
        first.watching.resume();
        const next = (await first.next()).compilation;
        assert.deepEqual(next.rebuildOrigin.causes.map(event => event.cause), [
          { kind: 'unknown' }, { kind: 'unknown' }, { kind: 'source', changed: ['changed.js'], removed: ['removed.js'] },
        ]);
        assert.equal(next.rebuildOrigin.throughRevision, 3);
        assert.equal(Reflect.set(next.rebuildOrigin.causes[2].cause.changed, '0', 'mutated'), false);
        assert.equal(Reflect.set(next, 'rebuildOrigin', undefined), false);
        assert.deepEqual(initial.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'initial' }]);
      } finally { await first.close(); }
      const restart = session(compiler);
      try {
        const next = (await restart.next()).compilation;
        assert.notEqual(next.rebuildOrigin.watchId, initial.rebuildOrigin.watchId);
        assert.equal(next.rebuildOrigin.throughRevision, 0);
        assert.deepEqual(initial.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'initial' }]);
      } finally { await restart.close(); }
    },
  },
  {
    description: 'keeps invalidations during a compilation for its successor and leaves additional passes unclassified',
    options,
    async build(context, compiler) {
      const compilations = [];
      compiler.hooks.thisCompilation.tap('Origins', compilation => {
        compilations.push(compilation);
        if (compilations.length === 1) {
          compilation.hooks.needAdditionalPass.tap('Origins', () => {
            compiler.watching.invalidate();
            return true;
          });
        }
      });
      const run = session(compiler);
      try {
        const stats = await run.next();
        assert.equal(compilations.length, 3);
        assert.deepEqual(compilations[0].rebuildOrigin.causes.map(event => event.cause), [{ kind: 'initial' }]);
        assert.equal(compilations[1].rebuildOrigin, undefined);
        assert.deepEqual(stats.compilation.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'unknown' }]);
        assert.equal(stats.compilation.rebuildOrigin.throughRevision, 1);
      } finally { await run.close(); }
    },
  },
  {
    description: 'reserves events observed during watchRun for the successor before the native compilation exists',
    options,
    async build(context, compiler) {
      let runs = 0;
      const compilations = [];
      compiler.hooks.watchRun.tapPromise('Origins', async () => {
        if (++runs === 1) {
          await setImmediate();
          compiler.watching.invalidate();
        }
      });
      compiler.hooks.thisCompilation.tap('Origins', compilation => compilations.push(compilation));
      const run = session(compiler);
      try {
        const stats = await run.next();
        assert.equal(compilations.length, 2);
        assert.deepEqual(compilations[0].rebuildOrigin.causes.map(event => event.cause), [{ kind: 'initial' }]);
        assert.deepEqual(stats.compilation.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'unknown' }]);
      } finally { await run.close(); }
    },
  },
  {
    description: 'includes source inputs arriving while the cache leaves idle in the same origin',
    options,
    async build(context, compiler) {
      const run = session(compiler);
      let release;
      try {
        await run.next();
        await setImmediate();
        let entered;
        const waiting = new Promise(resolve => { entered = resolve; });
        let hold = true;
        compiler.cache.hooks.endIdle.tapAsync('OriginCutoff', callback => {
          if (!hold) return callback();
          hold = false;
          release = callback;
          entered();
        });
        const inputs = [];
        compiler.hooks.watchRun.tap('OriginCutoff', () => {
          inputs.push({ changed: [...compiler.modifiedFiles], removed: [...compiler.removedFiles] });
        });
        run.watching.invalidateWithChangesAndRemovals(new Set(['first.js']), new Set());
        await waiting;
        run.watching.invalidateWithChangesAndRemovals(new Set(['second.js']), new Set(['removed.js']));
        release();
        release = undefined;
        const compilation = (await run.next()).compilation;
        assert.deepEqual(inputs, [{ changed: ['first.js', 'second.js'], removed: ['removed.js'] }]);
        assert.deepEqual(compilation.rebuildOrigin.causes.map(event => event.cause), [
          { kind: 'source', changed: ['first.js'], removed: [] },
          { kind: 'source', changed: ['second.js'], removed: ['removed.js'] },
        ]);
      } finally {
        release?.();
        await run.close();
      }
    },
  },
  {
    description: 'records the exact native lazy backend drain through HTTP including coalesced unknown and repeated activation',
    options() { return { ...options(), lazyCompilation: { entries: false, imports: true } }; },
    async build(context, compiler) {
      const snapshots = new Map();
      compiler.hooks.thisCompilation.tap('Snapshots', compilation => snapshots.set(compilation, compilation.rebuildOrigin));
      const middleware = lazyCompilationMiddleware(compiler);
      const server = createServer((request, response) => middleware(request, response));
      await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
      const url = `http://127.0.0.1:${server.address().port}/_rspack/lazy/trigger`;
      const post = async keys => {
        const response = await fetch(url, { method: 'POST', body: keys.join('\n') });
        assert.equal(response.status, 200);
        await response.text();
      };
      const run = session(compiler);
      try {
        const initial = (await run.next()).compilation;
        const key = [...initial.modules].find(module => module.identifier().startsWith('lazy-compilation-proxy|') && module.identifier().endsWith('lazy.js')).identifier();
        const secondKey = [...initial.modules].find(module => module.identifier().startsWith('lazy-compilation-proxy|') && module.identifier().endsWith('lazy2.js')).identifier();
        const events = [];
        compiler.hooks.watchInvalidation.tap('Origins', event => events.push(event));
        run.watching.suspend();
        await post([key, key]);
        await post(['opaque-second-key']);
        run.watching.invalidate();
        assert.deepEqual(events.map(event => event.cause), [
          { kind: 'lazy', keys: [key] }, { kind: 'lazy', keys: ['opaque-second-key'] }, { kind: 'unknown' },
        ]);
        run.watching.resume();
        const activated = (await run.next()).compilation;
        assert.deepEqual(activated.rebuildOrigin.causes.map(event => event.cause), [
          { kind: 'lazy', keys: [key] }, { kind: 'lazy', keys: ['opaque-second-key'] }, { kind: 'unknown' },
        ]);
        assert.deepEqual(activated.rebuildOrigin.consumedLazyKeys, [key, 'opaque-second-key']);
        assert.deepEqual(snapshots.get(activated).consumedLazyKeys, []);
        assert.deepEqual(initial.rebuildOrigin.consumedLazyKeys, []);
        assert.equal(Reflect.set(activated.rebuildOrigin.consumedLazyKeys, '0', 'injected'), false);
        assert.equal([...activated.modules].some(module => module.identifier().endsWith('lazy.js') && !module.identifier().startsWith('lazy-compilation-proxy|')), true);
        await post([key]);
        const repeated = (await run.next()).compilation;
        assert.deepEqual(repeated.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'lazy', keys: [key] }]);
        assert.deepEqual(repeated.rebuildOrigin.consumedLazyKeys, [key]);
        assert.deepEqual(activated.rebuildOrigin.consumedLazyKeys, [key, 'opaque-second-key']);
        let held = false;
        let release;
        let entered;
        const waiting = new Promise(resolve => { entered = resolve; });
        compiler.hooks.watchRun.tapPromise('HeldOrigin', async () => {
          if (!held) return;
          held = false;
          await new Promise(resolve => { release = resolve; entered(); });
        });
        const exact = [];
        compiler.hooks.thisCompilation.tap('HeldCompilations', compilation => exact.push(compilation));
        held = true;
        await post([key]);
        await waiting;
        const beforeDuplicate = events.length;
        await post([key]);
        assert.equal(events.length, beforeDuplicate);
        await post([key, secondKey]);
        release();
        const successor = (await run.next()).compilation;
        assert.equal(exact.length, 2);
        assert.deepEqual(exact[0].rebuildOrigin.causes.map(event => event.cause), [{ kind: 'lazy', keys: [key] }]);
        assert.deepEqual(exact[0].rebuildOrigin.consumedLazyKeys, [key]);
        assert.deepEqual(successor.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'lazy', keys: [secondKey] }]);
        assert.deepEqual(successor.rebuildOrigin.consumedLazyKeys, [secondKey]);
        assert.equal([...successor.modules].some(module => module.identifier().endsWith('lazy2.js') && !module.identifier().startsWith('lazy-compilation-proxy|')), true);
      } finally {
        await run.close();
        await new Promise(resolve => server.close(resolve));
      }
    },
  },
  {
    description: 'includes changes discovered by paused watcher getInfo before coalescing',
    options() { return { ...options(), experiments: { nativeWatcher: false } }; },
    async build(context, compiler) {
      const run = session(compiler);
      try {
        await run.next();
        await setImmediate();
        const watcher = run.watching.watcher;
        const getInfo = watcher.getInfo.bind(watcher);
        watcher.getInfo = () => ({ ...getInfo(), changes: new Set(['paused-change.js']), removals: new Set(['paused-remove.js']) });
        run.watching.invalidate();
        const stats = await run.next();
        assert.deepEqual(stats.compilation.rebuildOrigin.causes.map(event => event.cause), [
          { kind: 'unknown' }, { kind: 'source', changed: ['paused-change.js'], removed: ['paused-remove.js'] },
        ]);
      } finally { await run.close(); }
    },
  },
  {
    description: 'notifies on missing filenames and every coalesced watcher notification',
    options,
    async build(context, compiler) {
      const watch = compiler.watchFileSystem.watch.bind(compiler.watchFileSystem);
      let notify;
      compiler.watchFileSystem.watch = (...args) => {
        notify = args.at(-1);
        return watch(...args);
      };
      const events = [];
      compiler.hooks.watchInvalidation.tap('Origins', event => events.push(event.cause));
      const run = session(compiler);
      try {
        await run.next();
        await setImmediate();
        run.watching.suspend();
        notify(null, 1);
        notify('changed.js', 2);
        notify('changed-again.js', 3);
        run.watching.invalidateWithChangesAndRemovals(new Set(['changed.js', 'changed-again.js']), new Set());
        assert.deepEqual(events, [
          { kind: 'unknown' },
          { kind: 'source', changed: ['changed.js'], removed: [] },
          { kind: 'source', changed: ['changed-again.js'], removed: [] },
          { kind: 'source', changed: ['changed.js', 'changed-again.js'], removed: [] },
        ]);
        run.watching.resume();
        const stats = await run.next();
        assert.equal(stats.compilation.rebuildOrigin.throughRevision, 4);
        assert.deepEqual(stats.compilation.rebuildOrigin.causes[0].cause, { kind: 'unknown' });
      } finally { await run.close(); }
    },
  },
  {
    description: 'leaves one-shot compilations without watch origin and preserves origins on compilation errors',
    options,
    compiler(context, compiler) {
      compiler.hooks.thisCompilation.tap('OriginError', compilation => {
        compilation.hooks.processAssets.tap('OriginError', () => { compilation.errors.push(new Error('origin-error')); });
      });
    },
    async build(context, compiler) {
      const single = await new Promise((resolve, reject) => compiler.run((error, stats) => error ? reject(error) : resolve(stats)));
      assert.equal(single.compilation.rebuildOrigin, undefined);
      assert.equal(single.hasErrors(), true);
      const run = session(compiler);
      try {
        const watched = await run.next();
        assert.equal(watched.hasErrors(), true);
        assert.deepEqual(watched.compilation.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'initial' }]);
      } finally { await run.close(); }
    },
  },
];
