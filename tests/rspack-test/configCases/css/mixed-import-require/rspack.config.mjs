/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  entry: './index.js',
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  target: 'web',
};
