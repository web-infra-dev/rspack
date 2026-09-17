import { CssExtractRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  mode: 'development',
  devtool: false,
  output: {
    hotUpdateChunkFilename: '[id].hot-update.js',
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
          'css-loader',
        ],
      },
    ],
  },
  experiments: {
    css: false,
  },
  plugins: [
    new CssExtractRspackPlugin({
      filename: '[name].css',
    }),
  ],
};
