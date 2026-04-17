const rspack = require('@rspack/core');

const PLUGIN_NAME = 'rspack.SwcJsMinimizerRspackPlugin';

let updateIndex = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  mode: 'production',
  output: {
    chunkFilename: '[id].chunk.js',
  },
  optimization: {
    minimize: true,
    minimizer: [new rspack.SwcJsMinimizerRspackPlugin()],
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('MinimizePersistentCacheTest', (stats) => {
          const s = stats.toJson({
            all: false,
            assets: true,
            logging: 'log',
          });

          const jsAssets = s.assets.filter((a) => a.name?.endsWith('.js'));
          for (const asset of jsAssets) {
            expect(asset.info.minimized).toBe(true);
          }

          const logEntries = s.logging[PLUGIN_NAME]?.entries ?? [];
          const cacheLogEntry = logEntries.find(
            (e) =>
              e.message && e.message.includes('minimize persistent cache:'),
          );

          if (updateIndex === 5) {
            // HMR update with changed file content
            // Minimize persistent cache is not shared across in-memory rebuilds, so both assets are processed as new → all misses.
            expect(cacheLogEntry).toBeUndefined();
            return;
          }

          expect(cacheLogEntry).toBeTruthy();

          const match = cacheLogEntry.message.match(
            /minimize persistent cache: (\d+) hit, (\d+) miss/,
          );
          expect(match).toBeTruthy();

          const hits = parseInt(match[1], 10);
          const misses = parseInt(match[2], 10);

          if (updateIndex === 0) {
            // Cold build, cache is empty → all misses
            expect(hits).toBe(0);
            expect(misses).toBe(2);
          }
          if (updateIndex === 1) {
            // First hot build with same source content, all recovered from cache.
            expect(hits).toBe(2);
            expect(misses).toBe(0);
          }
          if (updateIndex === 2) {
            // Hot build with same source content.
            expect(hits).toBe(2);
            expect(misses).toBe(0);
          }
          if (updateIndex === 3) {
            // Third cold build with changed file content.
            // Async chunk unchanged → hit; entry chunk changed → miss.
            expect(hits).toBe(1);
            expect(misses).toBe(1);
          }
          if (updateIndex === 4) {
            // Cold restart. Async chunk still unchanged → hit.
            expect(hits).toBe(1);
            expect(misses).toBe(1);
          }

          updateIndex++;
        });
      },
    },
  ],
};
