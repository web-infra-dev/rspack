import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
        extractComments: {
          // Test case insensitivity - should match both 'LICENSE' and 'license'
          condition: /LICENSE/i,
        },
      }),
    ],
  },
});
