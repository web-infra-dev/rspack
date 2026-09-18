/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
  },
  target: 'web',
  module: {
    rules: [
      {
        test: /\.[cm]?js$/,
        parser: {
          worker: ['MyWorker()', '...'],
        },
      },
    ],
  },
};
