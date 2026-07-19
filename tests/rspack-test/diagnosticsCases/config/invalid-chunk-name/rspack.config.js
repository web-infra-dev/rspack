/** @type {import('@rspack/core').Configuration} */
module.exports = {
  context: __dirname,
  entry: './index.js',
  output: {
    chunkFilename: '[name].[contenthash].js',
    cssChunkFilename: '[name].[contenthash].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
