const fs = require('node:fs');
const path = require('node:path');

// Records the `change` events exposed by `WatchFileSystem.on` and by the
// watchpack-compatible `watcher` shim (the surface ts-checker-rspack-plugin
// reaches for), then writes them next to the bundle so the test bundle can
// assert on them. (`remove` classification is covered deterministically by the
// rspack_watcher Rust test, since native fs remove events are asynchronous and
// hard to assert reliably from a watch case.)
class RecordWatchEventsPlugin {
  apply(compiler) {
    const changed = [];
    const watcherChanged = [];

    compiler.hooks.afterEnvironment.tap('RecordWatchEventsPlugin', () => {
      const wfs = compiler.watchFileSystem;
      if (!wfs) {
        return;
      }

      // The standard `on` API is stable across watch cycles, so attach once.
      wfs.on?.('change', (file) => {
        changed.push(file);
      });

      // The `watcher` shim is recreated each cycle (mirroring watchpack), so
      // re-attach after every `watch()` — the same way ts-checker does.
      const originalWatch = wfs.watch.bind(wfs);
      wfs.watch = (...args) => {
        const watcher = originalWatch(...args);
        wfs.watcher?.on('change', (file) => {
          watcherChanged.push(file);
        });
        return watcher;
      };
    });

    compiler.hooks.done.tap('RecordWatchEventsPlugin', () => {
      fs.writeFileSync(
        path.join(compiler.outputPath, 'recorded-events.json'),
        JSON.stringify({ changed, watcherChanged }),
      );
    });
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  plugins: [new RecordWatchEventsPlugin()],
  watchOptions: {
    aggregateTimeout: 100,
  },
};
