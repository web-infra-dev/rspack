/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /.css$/,
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
