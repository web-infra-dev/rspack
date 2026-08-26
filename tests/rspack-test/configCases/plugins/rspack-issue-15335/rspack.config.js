const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        minimizerOptions: {
          compress: true,
          mangle: true,
        },
      }),
    ],
  },
};
