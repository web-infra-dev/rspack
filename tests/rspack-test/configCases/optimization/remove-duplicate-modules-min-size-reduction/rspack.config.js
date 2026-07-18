const rspack = require('@rspack/core');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: {
    a: './a',
    b: './b',
    c: './c',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
  },
  plugins: [
    new rspack.experiments.RemoveDuplicateModulesPlugin({
      minSizeReduction: 1000,
    }),
    {
      apply(compiler) {
        compiler.hooks.done.tap(
          'RemoveDuplicateModulesMinSizeReductionTest',
          (stats) => {
            const jsAssets = Object.keys(stats.compilation.assets).filter(
              (asset) => asset.endsWith('.js'),
            );
            expect(jsAssets).toHaveLength(3);
          },
        );
      },
    },
  ],
};
