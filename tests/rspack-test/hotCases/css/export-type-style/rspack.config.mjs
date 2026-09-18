/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'style',
        },
      },
    ],
  },
  experiments: {
    css: true,
  },
};
