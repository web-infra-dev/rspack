/** @type {import("@rspack/core").Configuration} */
export default {
  performance: {
    hints: false,
  },
  optimization: {
    splitChunks: {
      minSize: 1,
    },
    chunkIds: 'named',
  },
};
