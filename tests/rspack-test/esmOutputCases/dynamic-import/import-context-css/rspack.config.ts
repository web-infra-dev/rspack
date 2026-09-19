import { defineConfig } from '@rspack/cli';

import { CssExtractRspackPlugin } from '@rspack/core';
export default defineConfig({
  target: 'web',
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
      chunkFilename: '[name].css',
    }),
  ],
});
