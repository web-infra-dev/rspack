import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  cache: true,
  mode: 'development',
  entry: './index',
  module: {
    rules: [
      {
        test: /\.module.css$/,
        type: 'javascript/auto',
        use: [
          {
            loader: rspack.CssExtractRspackPlugin.loader,
            options: {
              emit: false,
              esModule: true,
            },
          },
          {
            loader: 'css-loader',
            options: {
              modules: {
                namedExport: false,
              },
            },
          },
          './loader.mjs',
        ],
      },
    ],
  },
  experiments: {
    css: false,
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: '[name].css',
      runtime: false,
    }),
  ],
});
