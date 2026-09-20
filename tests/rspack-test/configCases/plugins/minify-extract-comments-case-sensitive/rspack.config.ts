import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
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
});
