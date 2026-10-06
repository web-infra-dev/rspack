import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setImmediate } from 'node:timers/promises';

export default [false, true].map(nativeWatcher => {
  let directory;
  return {
    description: `reports real file edits and removals with nativeWatcher=${nativeWatcher}`,
    options() {
      directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-watch-origin-')));
      fs.writeFileSync(path.join(directory, 'index.js'), "import value from './value.js'; console.log(value);");
      fs.writeFileSync(path.join(directory, 'value.js'), 'export default 1;');
      return { context: directory, mode: 'development', entry: './index.js', experiments: { nativeWatcher } };
    },
    async build(context, compiler) {
      const results = [];
      const waiters = [];
      const watching = compiler.watch({ aggregateTimeout: 10 }, (error, stats) => {
        const result = { error, stats };
        if (waiters.length) waiters.shift()(result);
        else results.push(result);
      });
      const next = async () => {
        const { error, stats } = results.length ? results.shift() : await new Promise(resolve => waiters.push(resolve));
        if (error) throw error;
        return stats;
      };
      const value = path.join(directory, 'value.js');
      try {
        await next();
        await setImmediate();
        fs.writeFileSync(value, 'export default 2;');
        const edited = await next();
        assert.equal(edited.hasErrors(), false);
        assert.equal(edited.compilation.rebuildOrigin.causes.some(event => event.cause.kind === 'source' && event.cause.changed.includes(value)), true, JSON.stringify(edited.compilation.rebuildOrigin));
        await setImmediate();
        fs.unlinkSync(value);
        const removed = await next();
        assert.equal(removed.hasErrors(), true);
        assert.equal(removed.compilation.rebuildOrigin.causes.some(event => event.cause.kind === 'source' && event.cause.removed.includes(value)), true);
      } finally {
        await new Promise(resolve => watching.close(resolve));
        fs.rmSync(directory, { recursive: true, force: true });
      }
    },
  };
});
