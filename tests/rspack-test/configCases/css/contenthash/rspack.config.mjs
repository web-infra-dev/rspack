/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  output: {
    filename: 'bundle.[name].[contenthash].js',
    cssFilename: 'bundle.[name].[contenthash].css',
    chunkFilename: 'async.[name].[contenthash].js',
    cssChunkFilename: 'async.[name].[contenthash].css',
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
