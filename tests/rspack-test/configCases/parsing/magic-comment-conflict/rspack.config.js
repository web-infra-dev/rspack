/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  output: {
    chunkFilename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
  },
};
