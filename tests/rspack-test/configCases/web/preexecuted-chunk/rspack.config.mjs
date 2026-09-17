/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  output: {
    chunkFilename: '[name].js',
  },
  performance: {
    hints: false,
  },
  optimization: {
    chunkIds: 'named',
    minimize: false,
  },
};
