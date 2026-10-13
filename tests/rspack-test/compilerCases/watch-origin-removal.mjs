import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setImmediate } from 'node:timers/promises';
import { lazyCompilationMiddleware } from '@rspack/core';

let directory;
export default {
  description: 'reports native removals during a lazy watchRun beyond the compilation cutoff',
  options() {
    directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-origin-removal-')));
    fs.writeFileSync(path.join(directory, 'index.js'), "export const load = () => import('./lazy.js');");
    fs.writeFileSync(path.join(directory, 'lazy.js'), 'export default 1;');
    return {
      mode: 'development',
      devtool: false,
      context: directory,
      entry: './index.js',
      experiments: { nativeWatcher: true },
      lazyCompilation: { entries: false, imports: true },
    };
  },
  async build(context, compiler) {
    const events = [];
    compiler.hooks.watchInvalidation.tap('RemovalOrigin', event => events.push(event));
    const middleware = lazyCompilationMiddleware(compiler);
    const results = [];
    let receive;
    const watching = compiler.watch({ aggregateTimeout: 10000 }, (error, stats) => {
      const result = { error, stats };
      if (receive) {
        const resolve = receive;
        receive = undefined;
        resolve(result);
      } else results.push(result);
    });
    const next = async () => {
      const { error, stats } = results.length ? results.shift() : await new Promise(resolve => { receive = resolve; });
      if (error) throw error;
      return stats;
    };
    let release;
    try {
      const initial = await next();
      assert.equal(initial.hasErrors(), false, initial.toString());
      await setImmediate();
      const key = [...initial.compilation.modules].find(module => module.identifier().startsWith('lazy-compilation-proxy|')).identifier();
      let entered;
      const waiting = new Promise(resolve => { entered = resolve; });
      compiler.hooks.watchRun.tapPromise('RemovalOrigin', () => new Promise(resolve => {
        release = resolve;
        entered();
      }));
      await middleware(
        { url: '/_rspack/lazy/trigger', method: 'POST', body: key },
        { writeHead() {}, write() {}, end() {} },
      );
      await waiting;
      const file = path.join(directory, 'lazy.js');
      const removed = new Promise(resolve => compiler.watchFileSystem.once('remove', resolve));
      fs.unlinkSync(file);
      await removed;
      const deletion = events.at(-1);
      assert.deepEqual(deletion.cause, { kind: 'source', changed: [], removed: [file] });
      release();
      const stats = await next();
      assert.ok(stats.compilation.rebuildOrigin.throughRevision < deletion.revision);
      assert.deepEqual(stats.compilation.rebuildOrigin.causes.map(event => event.cause), [{ kind: 'lazy', keys: [key] }]);
    } finally {
      release?.();
      await new Promise(resolve => watching.close(resolve));
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
};
