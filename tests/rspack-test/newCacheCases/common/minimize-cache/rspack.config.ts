import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { getCacheStats } from '@rspack/test-tools';

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
        compiler.hooks.done.tapPromise(
          'MinimizePersistentCacheTest',
          async (stats) => {
            const s = stats.toJson({
              all: false,
              assets: true,
            });

            const jsAssets = s.assets!.filter((a) => a.name?.endsWith('.js'));
            for (const asset of jsAssets) {
              expect(asset.info.minimized).toBe(true);
            }

            const [cacheStats] = await getCacheStats(stats);
            const { hits, misses } = cacheStats[PLUGIN_NAME];

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
            if (updateIndex === 5) {
              // HMR keeps the async chunk cached; the entry and hot-update assets miss.
              expect(hits).toBe(1);
              expect(misses).toBe(2);
            }

            updateIndex++;
          },
        );
      },
    }),
  ],
});
