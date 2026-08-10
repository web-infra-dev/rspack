/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    filename: 'bundle.js',
    chunkFilename: '[name].bundle.js',
  },
  optimization: {
    splitChunks: false,
    sideEffects: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
  stats: {
    preset: 'verbose',
    logging: 'verbose',
    loggingDebug: [/codeSplittingCache/, /incremental/],
  },
};
