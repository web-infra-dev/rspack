/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    path: 'node-commonjs path',
  },
  module: {
    rules: [
      {
        test: /\.scss$/,
        use: [{ loader: 'sass-loader' }],
        type: 'css/module',
      },
    ],
  },
};
