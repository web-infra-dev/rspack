import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const PLUGIN_NAME = 'rspack.SwcJsMinimizerRspackPlugin';

let updateIndex = 0;

export default defineConfig({
  context: import.meta.dirname,
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
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('MinimizePersistentCacheTest', (stats) => {
          const s = stats.toJson({
            all: false,
            assets: true,
            cacheInfo: true,
          });

          const jsAssets = s.assets!.filter((a) => a.name?.endsWith('.js'));
          for (const asset of jsAssets) {
            expect(asset.info.minimized).toBe(true);
          }

          const cacheStats = s.cacheInfo!;
          const { hits, misses } =
            cacheStats[`${compiler.compilerPath}|${PLUGIN_NAME}`];

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
            // Both assets carry the changed compilation hash.
            expect(hits).toBe(0);
            expect(misses).toBe(2);
          }
          if (updateIndex === 4) {
            // The changed compilation hash also invalidates the async asset.
            expect(hits).toBe(0);
            expect(misses).toBe(2);
          }
          if (updateIndex === 5) {
            // The async asset, entry and hot-update asset all change.
            expect(hits).toBe(0);
            expect(misses).toBe(3);
          }

          updateIndex++;
        });
      },
    }),
  ],
});
