import { defineConfig, definePlugin } from '@rspack/cli';

const PLUGIN_NAME = 'rspack.SourceMapDevToolPlugin';

let updateIndex = 0;
let firstAssets: Array<[string, string]>;

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
        compiler.hooks.done.tap('SourceMapPersistentCacheTest', (stats) => {
          const assets = stats.compilation
            .getAssets()
            .map(({ name, source }) => {
              expect(source.map()).toBeNull();
              return [name, source.buffer().toString('base64')] as [
                string,
                string,
              ];
            })
            .sort(([a], [b]) => a.localeCompare(b));
          if (updateIndex === 0) firstAssets = assets;
          if (updateIndex === 1) expect(assets).toEqual(firstAssets);
          const cacheStats = stats.toJson({
            all: false,
            cacheInfo: true,
          }).cacheInfo!;
          const { hits, misses } =
            cacheStats[`${compiler.compilerPath}|${PLUGIN_NAME}`];

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
        });
      },
    }),
  ],
});
