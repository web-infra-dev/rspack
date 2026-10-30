import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';

// Dumps `compiler.fileTimestamps` / `contextTimestamps` and the times API of
// `WatchFileSystem` (and its `watcher` shim) next to the bundle for the test
// to assert on. Runs under both the watchpack and native watcher projects.
class RecordWatchTimesPlugin {
  apply(compiler: Compiler) {
    const dump = <K, V>(entries: ReadonlyMap<K, V> | undefined) =>
      entries ? Array.from(entries.entries()) : null;

    compiler.hooks.done.tap('RecordWatchTimesPlugin', () => {
      // Both watcher implementations expose the compatibility shim at runtime.
      const wfs = compiler.watchFileSystem as Compiler['watchFileSystem'] & {
        watcher?: { getTimes?(): Record<string, number> };
      };
      const collectedFiles = new Map();
      const collectedContexts = new Map();
      wfs?.collectTimeInfoEntries?.(collectedFiles, collectedContexts);

      fs.writeFileSync(
        path.join(compiler.outputPath, 'watch-times.json'),
        JSON.stringify({
          fileTimestamps: dump(compiler.fileTimestamps),
          contextTimestamps: dump(compiler.contextTimestamps),
          times: wfs?.getTimes?.() ?? {},
          timeInfoEntries: dump(wfs?.getTimeInfoEntries?.()) ?? [],
          collectedFiles: dump(collectedFiles),
          collectedContexts: dump(collectedContexts),
          watcherTimes: wfs?.watcher?.getTimes?.() ?? {},
        }),
      );
    });
  }
}

export default defineConfig({
  plugins: [new RecordWatchTimesPlugin()],
  watchOptions: {
    aggregateTimeout: 100,
  },
});
