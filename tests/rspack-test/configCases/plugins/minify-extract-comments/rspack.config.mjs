import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        extractComments: true,
      }),
    ],
  },
};
