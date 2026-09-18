/** @type {import('@rspack/core').Configuration} */
export default {
  entry: './index',
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['./test-loader.mjs'],
      },
    ],
  },
  stats: {
    all: false,
    loggingDebug: [/TestLoader/],
  },
};
