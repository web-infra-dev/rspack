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
          drop_console: true,
        },
      },
    }),
  ],
};
