import fs from 'node:fs';
import path from 'node:path';

// Records what the watcher reports about watched-path timestamps: the tables
// webpack receives through the `watch` callback (`compiler.fileTimestamps` /
// `contextTimestamps`) and the watchpack-compatible times API on
// `WatchFileSystem` (plus the same API on the `watcher` shim, the surface
// plugins like ts-checker-rspack-plugin reach for). Written next to the bundle
// so the test bundle can assert on it. Runs against both watcher backends:
// watchpack (`Watch.part*`) and the native watcher (`NativeWatcher.part*`).
class RecordWatchTimesPlugin {
  apply(compiler) {
    const dump = (entries) => (entries ? Array.from(entries.entries()) : null);

    compiler.hooks.done.tap('RecordWatchTimesPlugin', () => {
      const wfs = compiler.watchFileSystem;
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

/** @type {import('@rspack/core').Configuration} */
export default {
  plugins: [new RecordWatchTimesPlugin()],
  watchOptions: {
    aggregateTimeout: 100,
  },
};
