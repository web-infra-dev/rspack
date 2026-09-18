import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
