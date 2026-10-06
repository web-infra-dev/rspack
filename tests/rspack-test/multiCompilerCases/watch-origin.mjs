import assert from 'node:assert/strict';
import path from 'node:path';
import { createServer } from 'node:http';
import { lazyCompilationMiddleware } from '@rspack/core';

export default {
  description: 'keeps MultiCompiler dispatch lazy-only while manual and dependency invalidations remain unknown',
  options() {
    const context = path.resolve(import.meta.dirname, '../compilerCases/watch-origin');
    return [
      { name: 'parent', context, mode: 'development', entry: './index.js', devtool: false, lazyCompilation: { entries: false, imports: true } },
      { name: 'child', dependencies: ['parent'], context, mode: 'development', entry: './index.js', devtool: false, lazyCompilation: { entries: false, imports: true } },
    ];
  },
  async build(context, compiler) {
    const middleware = lazyCompilationMiddleware(compiler);
    const server = createServer((req, res) => middleware(req, res, () => { res.writeHead(404); res.end(); }));
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const post = async (index, keys) => {
      const response = await fetch(`http://127.0.0.1:${server.address().port}/_rspack/lazy/trigger__${index}`, { method: 'POST', body: keys.join('\n') });
      assert.equal(response.status, 200);
      await response.text();
    };
    const results = [];
    const waiters = [];
    const watching = compiler.watch({}, (error, stats) => {
      const result = { error, stats };
      if (waiters.length) waiters.shift()(result);
      else results.push(result);
    });
    const next = async () => {
      const { error, stats } = results.length ? results.shift() : await new Promise(resolve => waiters.push(resolve));
      if (error) throw error;
      return stats;
    };
    try {
      const initial = await next();
      for (const stats of initial.stats) assert.deepEqual(stats.compilation.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'initial' }]);
      const [parent, child] = compiler.compilers;
      const key = [...initial.stats[1].compilation.modules].find(module => module.identifier().startsWith('lazy-compilation-proxy|')).identifier();
      await post(1, [key]);
      const lazy = await next();
      assert.equal(lazy.stats.length, 1);
      assert.deepEqual(lazy.stats[0].compilation.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'lazy', keys: [key] }]);
      assert.deepEqual(lazy.stats[0].compilation.rebuildOrigin.consumedLazyKeys, [key]);
      let mixNext = true;
      child.hooks.watchInvalidation.tap('CoalesceManual', event => {
        if (mixNext && event.cause.kind === 'lazy') {
          mixNext = false;
          child.watching.invalidate();
        }
      });
      await post(1, [key]);
      const mixed = await next();
      assert.deepEqual(mixed.stats[0].compilation.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'lazy', keys: [key] }, { kind: 'unknown' }]);
      parent.watching.invalidateWithChangesAndRemovals(new Set(['parent-source.js']), new Set());
      const dependency = await next();
      assert.deepEqual(dependency.stats.map(stats => [stats.compilation.name, stats.compilation.rebuildOrigin.causes.map(event => event.cause)]), [
        ['parent', [{ kind: 'source', changed: ['parent-source.js'], removed: [] }]],
        ['child', [{ kind: 'unknown' }]],
      ]);
    } finally {
      await new Promise(resolve => watching.close(resolve));
      await new Promise(resolve => server.close(resolve));
    }
  },
};
