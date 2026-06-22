/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './chunk1.js': 'commonjs ./chunk1.js',
  },
  target: 'web',
  output: {
    chunkFilename: '[name].js',
    charset: false,
  },
  performance: {
    hints: false,
  },
  optimization: {
    chunkIds: 'named',
    minimize: false,
  },
};
