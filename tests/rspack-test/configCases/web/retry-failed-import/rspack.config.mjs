/** @type {import("@rspack/core").Configuration} */
export default {
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
