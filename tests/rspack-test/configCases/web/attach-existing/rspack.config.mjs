/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    './the-chunk.js': 'commonjs ./the-chunk.js',
  },
  target: 'web',
  output: {
    chunkFilename: '[name].js',
    uniqueName: 'my "app"',
  },
  performance: {
    hints: false,
  },
  optimization: {
    chunkIds: 'named',
    minimize: false,
  },
};
