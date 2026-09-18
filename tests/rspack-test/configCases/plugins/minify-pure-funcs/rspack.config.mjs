import { rspack } from '@rspack/core';

/**
 * @type {import("@rspack/core").Configuration}
 */
export default {
  optimization: {
    minimize: true,
  },
  plugins: [
    new rspack.SwcJsMinimizerRspackPlugin({
      minimizerOptions: {
        compress: {
          pure_funcs: ['__logger.error', '__logger.warn'],
        },
      },
    }),
  ],
};
