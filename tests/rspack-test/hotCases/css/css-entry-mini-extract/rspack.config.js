'use strict';

const { CssExtractRspackPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  entry: {
    'css-entry': './entry.css',
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [
          {
            loader: CssExtractRspackPlugin.loader,
          },
          {
            loader: 'css-loader',
            options: {
              esModule: true,
              modules: {
                namedExport: false,
                localIdentName: '[name]',
              },
            },
          },
        ],
      },
    ],
  },
  output: {
    filename: '[name].js',
    cssChunkFilename: '[name].css',
  },
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        styles: {
          type: 'css/mini-extract',
          enforce: true,
        },
      },
    },
  },
  target: 'web',
  node: {
    __dirname: false,
  },
  plugins: [new CssExtractRspackPlugin({ filename: 'css-entry.css' })],
};
