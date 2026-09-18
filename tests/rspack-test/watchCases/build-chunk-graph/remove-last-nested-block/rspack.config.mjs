/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
};
