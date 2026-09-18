/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /wrapper-style\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'style',
        },
      },
      {
        test: /base\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
    ],
  },
  experiments: {
    css: true,
  },
};
