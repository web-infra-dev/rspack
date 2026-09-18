/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    chunkFilename: '[id].[contenthash].js',
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
};
