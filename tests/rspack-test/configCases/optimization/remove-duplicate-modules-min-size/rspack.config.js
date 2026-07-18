const rspack = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: {
    a: './a',
    b: './b',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
  },
  plugins: [
    new rspack.experiments.RemoveDuplicateModulesPlugin({
      minSize: 1000,
    }),
    {
      apply(compiler) {
        compiler.hooks.done.tap(
          'RemoveDuplicateModulesMinSizeTest',
          (stats) => {
            const jsAssets = Object.keys(stats.compilation.assets)
              .filter((asset) => asset.endsWith('.js'))
              .sort();
            expect(jsAssets).toEqual(['a.js', 'b.js']);
          },
        );
      },
    },
  ],
};
