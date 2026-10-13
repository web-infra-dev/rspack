import assert from 'node:assert/strict';
import path from 'node:path';
import { setImmediate } from 'node:timers/promises';

export default [false, true].map(wrapUndelayed => ({
  description: wrapUndelayed
    ? 'preserves wrapped undelayed watcher callbacks'
    : 'reports repeated changes and removals before aggregation',
  options() {
    return { experiments: { nativeWatcher: false } };
  },
  async build(context, compiler) {
    const events = [];
    const invalidations = [];
    let wrapperCalls = 0;
    if (wrapUndelayed) {
      const watch = compiler.watchFileSystem.watch.bind(compiler.watchFileSystem);
      compiler.watchFileSystem.watch = (...args) => {
        const notify = args[6];
        args[6] = (_file, time) => {
          wrapperCalls++;
          notify(null, time);
        };
        return watch(...args);
      };
    }
    compiler.hooks.watchInvalidation.tap('SourceEvents', event => events.push(event));
    compiler.hooks.invalid.tap('SourceEvents', filename => invalidations.push(filename));
    let watching;
    try {
      await new Promise((resolve, reject) => {
        watching = compiler.watch({ aggregateTimeout: 10000 }, error => error ? reject(error) : resolve());
      });
      await setImmediate();
      watching.suspend();
      const watcher = compiler.watchFileSystem.watcher;
      const file = path.join(compiler.context, 'source-event.js');
      watcher._onChange(file, 1, file, 'change');
      watcher._onChange(file, 2, file, 'change');
      watcher._onRemove(file, file, 'remove');
      assert.deepEqual(events.map(event => event.cause), wrapUndelayed ? [
        { kind: 'unknown' },
      ] : [
        { kind: 'source', changed: [file], removed: [] },
        { kind: 'source', changed: [file], removed: [] },
        { kind: 'source', changed: [], removed: [file] },
      ]);
      assert.deepEqual(events.map(event => event.revision), wrapUndelayed ? [1] : [1, 2, 3]);
      assert.deepEqual(invalidations, [wrapUndelayed ? null : file]);
      assert.equal(wrapperCalls, wrapUndelayed ? 1 : 0);
    } finally {
      if (watching) await new Promise(resolve => watching.close(resolve));
    }
  },
}));
