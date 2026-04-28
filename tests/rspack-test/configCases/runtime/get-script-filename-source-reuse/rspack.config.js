/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  output: {
    filename: '[name].js',
    chunkFilename: 'chunks/[name].[contenthash:8].js',
  },
  optimization: {
    chunkIds: 'named',
    runtimeChunk: {
      name: 'runtime',
    },
  },
};
