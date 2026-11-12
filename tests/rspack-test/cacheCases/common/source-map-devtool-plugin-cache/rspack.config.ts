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
      test: /\.chunk\.js$/,
      filename: '[file].map',
      append: () => `\n//# sourceMappingURL=[url]?build=${updateIndex}`,
    }),
    new rspack.SourceMapDevToolPlugin({
      test: /^bundle\.js$/,
      noSources: true,
    }),
    new rspack.SourceMapDevToolPlugin({
      test: /^bundle\.js$/,
      noSources: true,
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('CheckDevtoolSource', (compilation) => {
          compilation.hooks.afterProcessAssets.tap('CheckDevtoolSource', () => {
            for (const asset of compilation.getAssets()) {
              if (!asset.name.endsWith('.js')) continue;
              expect(asset.source.map()).toBeNull();
              const text = asset.source.source().toString();
              expect(text.match(/sourceMappingURL=/g)).toHaveLength(1);
              if (asset.name === 'bundle.js') {
                expect(text).toContain(
                  'sourceMappingURL=data:application/json',
                );
              } else {
                expect(text).toContain(
                  `sourceMappingURL=${asset.name}.map?build=${updateIndex}`,
                );
              }
            }
          });
        });
        compiler.hooks.done.tap('SourceMapPersistentCacheTest', (stats) => {
          const s = stats.toJson({
            all: false,
            logging: 'verbose',
          });

          const logEntries = s.logging?.[PLUGIN_NAME]?.entries ?? [];
          const counts = logEntries
            .filter((entry) => entry.type === 'cache')
            .map((entry) => entry.message?.match(/\((\d+)\/(\d+)\)$/)?.[0]);

          // Each instance sees one asset. The last receives the already inlined bundle.
          expect(counts).toEqual([
            updateIndex === 0 ? '(0/1)' : '(1/1)',
            updateIndex === 1 ? '(1/1)' : '(0/1)',
            '(0/1)',
          ]);
          updateIndex++;
        });
      },
    }),
  ],
});
