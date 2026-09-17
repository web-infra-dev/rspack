/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /text\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
      {
        test: /stylesheet\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'css-style-sheet',
        },
      },
    ],
  },
  experiments: {
    css: true,
  },
};
