/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: true,
  },
  incremental: {
    buildChunkGraph: true,
  },
};
