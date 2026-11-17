import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const PLUGIN_NAME = 'rspack.SourceMapDevToolPlugin';

let updateIndex = 0;

export default defineConfig({
  context: import.meta.dirname,
  devtool: false,
  output: {
    filename: 'bundle.js',
    chunkFilename: '[name].chunk.js',
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      filename: '[file].map',
      append: () => `\n//# sourceMappingURL=[url]?build=${updateIndex}`,
      moduleFilenameTemplate: 'shared-name',
      fallbackModuleFilenameTemplate: '[resource-path]',
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('CheckDevtoolSource', (compilation) => {
          compilation.hooks.afterProcessAssets.tap('CheckDevtoolSource', () => {
            const contents = new Map<string, string>();
            for (const asset of compilation.getAssets()) {
              if (asset.name.endsWith('.map')) {
                const map = JSON.parse(asset.source.source().toString());
                for (const [i, name] of map.sources.entries()) {
                  if (contents.has(name)) {
                    expect(map.sourcesContent[i]).toBe(contents.get(name));
                  }
                  contents.set(name, map.sourcesContent[i]);
                }
              }
              if (!asset.name.endsWith('.js')) continue;
              expect(asset.source.map()).toBeNull();
              const text = asset.source.source().toString();
              expect(text.match(/sourceMappingURL=/g)).toHaveLength(1);
              expect(text).toContain(
                `sourceMappingURL=${asset.name}.map?build=${updateIndex}`,
              );
            }
          });
        });
        compiler.hooks.done.tap('SourceMapPersistentCacheTest', (stats) => {
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
