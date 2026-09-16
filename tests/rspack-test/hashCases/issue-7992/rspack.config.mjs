/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './src/index.js',
  devtool: false,
  output: {
    filename: 'main.js',
    assetModuleFilename: '[name].[contenthash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(svg)$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    realContentHash: true,
  },
};
