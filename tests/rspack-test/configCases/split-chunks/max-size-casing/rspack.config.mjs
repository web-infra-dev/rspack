/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      hidePathInfo: false,
      minSize: 50,
      maxSize: 100,
    },
  },
};
