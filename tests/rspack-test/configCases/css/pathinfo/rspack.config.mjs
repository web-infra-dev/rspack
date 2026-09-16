/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  devtool: false,
  output: {
    pathinfo: true,
    cssChunkFilename: '[name].[chunkhash].css',
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
