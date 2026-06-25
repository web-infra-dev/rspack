'use strict';

const MiniCssExtractPlugin = require('@rspack/core').CssExtractRspackPlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  devtool: false,
  target: 'web',
  entry: './index.js',
  mode: 'development',
  optimization: {
    concatenateModules: false,
  },
  experiments: {
    css: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [
          {
            loader: MiniCssExtractPlugin.loader,
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
  plugins: [
    new MiniCssExtractPlugin({
      filename: '[name].css',
    }),
  ],
  node: {
    __dirname: false,
    __filename: false,
  },
};
