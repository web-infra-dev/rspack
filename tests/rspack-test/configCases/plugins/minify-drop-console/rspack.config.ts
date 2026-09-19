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
          drop_console: true,
        },
      },
    }),
  ],
});
