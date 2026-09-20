import { defineConfig } from '@rspack/cli';

import { SwcJsMinimizerRspackPlugin } from '@rspack/core';

export default defineConfig({
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
});
