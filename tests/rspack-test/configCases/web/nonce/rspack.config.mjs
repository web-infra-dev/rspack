/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  output: {
    chunkFilename: '[name].js',
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
