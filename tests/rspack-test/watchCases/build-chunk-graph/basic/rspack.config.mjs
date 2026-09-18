/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    splitChunks: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
};
