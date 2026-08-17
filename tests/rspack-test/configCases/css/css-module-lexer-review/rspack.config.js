/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  mode: 'development',
  devtool: false,
  output: {
    cssFilename: 'bundle0.css',
  },
  module: {
    generator: {
      'css/module': {
        exportsOnly: false,
        localIdentName: '[name]-[local]',
      },
    },
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
      },
    ],
  },
};
