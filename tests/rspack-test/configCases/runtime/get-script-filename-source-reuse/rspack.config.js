/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  target: 'node',
  entry: './index.js',
  mode: 'development',
  devtool: false,
  output: {
    filename: '[name].js',
    chunkFilename: 'chunks/[name].[contenthash:8].js',
  },
  optimization: {
    minimize: false,
    chunkIds: 'named',
    runtimeChunk: {
      name: 'runtime',
    },
  },
};
