/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    bundle0: './a',
    bundle1: './b',
  },
  optimization: {
    // flagIncludedChunks: false,
    chunkIds: 'named',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[id].[chunkhash].js',
  },
};
