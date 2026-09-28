import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.(jsx?|tsx?)$/,
        use: [
          {
            loader: 'builtin:swc-loader',
          },
        ],
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.afterEmit.tap('PLUGIN', (compilation) => {
          const sourceMap = JSON.parse(
            compilation.assets['bundle0.js.map'].source().toString(),
          );
          let sourceUrl = (source: string) => `webpack:///${source}`;
          if (compiler.options.experiments?.runtimeMode === 'rspack') {
            sourceUrl = (source: string) => `rspack:///${source}`;
          }
          expect(sourceMap.sources).toEqual(
            expect.arrayContaining([
              sourceUrl('./node_modules/lib-with-source-map/main.js'),
              sourceUrl('./index.js'),
            ]),
          );
        });
      },
    }),
  ],
});
