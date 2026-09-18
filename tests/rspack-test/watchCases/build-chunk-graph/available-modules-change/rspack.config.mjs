/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    splitChunks: false,
    sideEffects: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
};
