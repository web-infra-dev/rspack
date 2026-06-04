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
    generator: {
      'css/module': {
        localIdentName: '[local]',
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
      },
    ],
  },
  experiments: {
    css: true,
  },
};
