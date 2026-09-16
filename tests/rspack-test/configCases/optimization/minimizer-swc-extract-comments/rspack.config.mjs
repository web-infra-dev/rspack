import { SwcJsMinimizerRspackPlugin } from '@rspack/core';

/**
 * @type {import("@rspack/core").Configuration}
 */
export default {
  optimization: {
    minimize: true,
    minimizer: [
      new SwcJsMinimizerRspackPlugin({
        extractComments: {},
        minimizerOptions: {
          format: {
            comments: 'all',
          },
        },
      }),
    ],
  },
};
