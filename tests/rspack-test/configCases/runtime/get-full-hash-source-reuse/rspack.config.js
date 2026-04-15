/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  target: 'node',
  entry: './index.js',
  mode: 'development',
  devtool: false,
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
    runtimeChunk: {
      name: 'runtime',
    },
  },
};
