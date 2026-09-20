import { defineConfig } from '@rspack/cli';

import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    light: { import: './light.js', layer: 'light' },
    dark: { import: './dark.js', layer: 'dark' },
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
  module: {
    rules: [
      {
        test: /\.less$/i,
        type: 'javascript/auto',
        oneOf: [
          {
            issuerLayer: 'light',
            use: [
              CssExtractRspackPlugin.loader,
              'css-loader',
              {
                loader: 'less-loader',
                options: {
                  additionalData: '@color: white;',
                },
              },
            ],
          },
          {
            issuerLayer: 'dark',
            use: [
              CssExtractRspackPlugin.loader,
              'css-loader',
              {
                loader: 'less-loader',
                options: {
                  additionalData: '@color: black;',
                },
              },
            ],
          },
        ],
      },
    ],
  },
});
