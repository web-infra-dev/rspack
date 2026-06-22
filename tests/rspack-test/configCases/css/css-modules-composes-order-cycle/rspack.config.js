/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
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
