/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  output: {
    pathinfo: true,
  },
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
      },
    ],
  },
};
