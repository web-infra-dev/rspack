/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    crossOriginLoading: 'use-credentials',
  },
  entry: './index.js',
  target: 'web',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
      },
    ],
  },
};
