/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  output: {
    chunkFilename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
  },
};
