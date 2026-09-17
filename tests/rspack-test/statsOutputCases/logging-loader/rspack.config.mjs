/** @type {import('@rspack/core').Configuration} */
export default {
  entry: './index',
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['./test-loader.js'],
      },
    ],
  },
  stats: {
    all: false,
    loggingDebug: [/TestLoader/],
  },
};
