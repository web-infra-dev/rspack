/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  mode: 'development',
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
