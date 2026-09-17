import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
          './loader.js',
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
};
