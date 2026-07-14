/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    splitChunks: false,
    sideEffects: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
  stats: {
    preset: "verbose",
    logging: "verbose",
    loggingDebug: [/codeSplittingCache/, /incremental/],
  },
};
