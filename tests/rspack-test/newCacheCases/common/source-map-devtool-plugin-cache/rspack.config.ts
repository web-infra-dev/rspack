import { defineConfig, definePlugin } from '@rspack/cli';
import { getCacheStats } from '@rspack/test-tools';

const PLUGIN_NAME = 'rspack.SourceMapDevToolPlugin';

let updateIndex = 0;

export default defineConfig({
  context: import.meta.dirname,
  devtool: 'source-map',
  output: {
    filename: 'bundle.js',
    chunkFilename: '[name].chunk.js',
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tapPromise(
          'SourceMapPersistentCacheTest',
          async (stats) => {
            const [cacheStats] = await getCacheStats(stats);
            const { hits, misses } = cacheStats[PLUGIN_NAME];

            if (updateIndex === 0) {
              expect(hits).toBe(0);
              expect(misses).toBe(2);
            }
            if (updateIndex === 1) {
              expect(hits).toBe(2);
              expect(misses).toBe(0);
            }
            if (updateIndex === 2) {
              expect(hits).toBe(1);
              expect(misses).toBe(1);
            }

            updateIndex++;
          },
        );
      },
    }),
  ],
});
