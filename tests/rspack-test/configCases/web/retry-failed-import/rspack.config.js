/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './the-chunk.js': 'commonjs ./the-chunk.js',
  },
  target: 'web',
  output: {
    chunkFilename: '[name].js',
  },
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
  },
};
