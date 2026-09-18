/** @type {import("../../../../").Configuration} */
export default {
  output: {
    chunkFilename: 'chunk-[name].js',
  },
  optimization: {
    chunkIds: 'named',
  },
};
