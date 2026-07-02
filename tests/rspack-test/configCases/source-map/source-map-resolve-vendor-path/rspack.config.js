/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
    {
      apply(compiler) {
        compiler.hooks.afterEmit.tap('PLUGIN', (compilation) => {
          const sourceMap = JSON.parse(
            compilation.assets['bundle0.js.map'].source(),
          );
          let sourceUrl = (source) => `webpack:///${source}`;
          if (compiler.options.experiments?.runtimeMode === 'rspack') {
            sourceUrl = (source) => `rspack:///${source}`;
          }
          expect(sourceMap.sources).toEqual(
            expect.arrayContaining([
              sourceUrl('./node_modules/lib-with-source-map/main.js'),
              sourceUrl('./index.js'),
            ]),
          );
        });
      },
    },
  ],
};
