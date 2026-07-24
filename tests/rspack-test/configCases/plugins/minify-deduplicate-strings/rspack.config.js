const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        minimizerOptions: {
          compress: false,
          mangle: false,
        },
      }),
    ],
  },
  plugins: [
    new rspack.DefinePlugin({
      __STRING_DEDUPLICATION_FILLER__: JSON.stringify('x'.repeat(5_000)),
    }),
  ],
};
