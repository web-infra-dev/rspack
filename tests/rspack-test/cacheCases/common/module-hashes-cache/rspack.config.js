const PLUGIN_NAME = 'rspack.persistentCache';

let updateIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  mode: 'development',
  cache: {
    type: 'persistent',
  },
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
    usedExports: false,
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('ModuleHashesPersistentCacheTest', (stats) => {
          const s = stats.toJson({
            all: false,
            logging: 'verbose',
          });

          const logEntries = s.logging[PLUGIN_NAME]?.entries ?? [];
          const cacheLogEntry = logEntries.find(
            (e) =>
              e.type === 'cache' &&
              e.message &&
              e.message.startsWith('module hashes persistent cache:'),
          );

          expect(cacheLogEntry).toBeTruthy();

          const match = cacheLogEntry.message.match(
            /module hashes persistent cache: [\d.]+% \((\d+)\/(\d+)\)/,
          );
          expect(match).toBeTruthy();

          const hits = parseInt(match[1], 10);
          const total = parseInt(match[2], 10);
          const misses = total - hits;

          if (updateIndex === 0) {
            // Cold build, cache is empty.
            expect(hits).toBe(0);
            expect(misses).toBe(2);
          }
          if (updateIndex === 1) {
            // Cold restart with the same source content.
            // JS modules recovered from the make cache are rebuilt in this path,
            // so module hashes are still recorded as cache misses for now.
            expect(hits).toBe(0);
            expect(misses).toBe(2);
          }

          updateIndex++;
        });
      },
    },
  ],
};
