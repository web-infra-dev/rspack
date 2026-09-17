/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
  },
  optimization: {
    chunkIds: 'named',
  },
};
