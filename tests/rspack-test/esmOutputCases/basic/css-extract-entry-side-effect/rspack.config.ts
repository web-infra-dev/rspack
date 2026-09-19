import { defineConfig } from '@rspack/cli';

import { CssExtractRspackPlugin } from '@rspack/core';
export default defineConfig({
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'javascript/auto',
        sideEffects: true,
        use: [
          {
            loader: CssExtractRspackPlugin.loader,
            options: {
              exportsOnly: false,
            },
          },
          'css-loader',
        ],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
});
