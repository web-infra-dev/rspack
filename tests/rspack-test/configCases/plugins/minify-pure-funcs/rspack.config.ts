import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
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
});
