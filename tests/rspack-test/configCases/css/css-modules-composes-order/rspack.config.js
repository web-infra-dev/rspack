/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  mode: 'development',
  devtool: false,
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.modules\.css$/,
        type: 'css/module',
      },
    ],
  },
  experiments: {
    css: true,
  },
};
