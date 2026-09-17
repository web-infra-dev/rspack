import { rspack } from '@rspack/core';

/**@type {import("@rspack/core").Configuration}*/
export default {
  output: {
    filename: 'bundle0.js?hash=[contenthash]',
  },
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        extractComments: true,
      }),
    ],
  },
};
