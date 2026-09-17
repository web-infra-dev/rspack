/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  module: {
    rules: [
      {
        test: /\.png/,
        type: 'asset/resource',
      },
    ],
  },
  stats: {
    all: true,
    timings: false,
    builtAt: false,
    version: false,
  },
};
