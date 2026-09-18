/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    splitChunks: {
      minSize: 1,
      chunks: 'all',
      cacheGroups: {
        foo: {
          minSize: 0,
        },
      },
    },
  },
};
