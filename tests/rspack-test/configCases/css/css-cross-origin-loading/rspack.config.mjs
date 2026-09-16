/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    crossOriginLoading: 'anonymous',
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
