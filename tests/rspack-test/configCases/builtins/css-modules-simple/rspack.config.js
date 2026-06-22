/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/,
        type: 'css/module',
      },
    ],
  },
};
