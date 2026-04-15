/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: {
      name: 'runtime',
    },
  },
};
