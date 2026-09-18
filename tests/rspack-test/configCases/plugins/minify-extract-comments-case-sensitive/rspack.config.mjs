import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        extractComments: {
          // Test case sensitivity - this should NOT match 'license' (lowercase)
          condition: /LICENSE/,
        },
      }),
    ],
  },
};
