/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  target: 'node',
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  optimization: {
    concatenateModules: false,
    innerGraph: true,
    minimize: false,
    providedExports: true,
    sideEffects: true,
    splitChunks: false,
    usedExports: true,
  },
  incremental: {
    buildChunkGraph: true,
  },
  stats: {
    loggingDebug: /codeSplittingCache/,
  },
};
