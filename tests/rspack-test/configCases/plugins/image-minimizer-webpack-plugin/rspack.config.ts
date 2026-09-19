import { defineConfig } from '@rspack/cli';

import ImageMinimizerPlugin from 'image-minimizer-webpack-plugin';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    minimize: true,
    minimizer: [
      new ImageMinimizerPlugin({
        minimizer: {
          implementation: async (original) => original,
        },
        loader: false,
      }),
    ],
  },
});
