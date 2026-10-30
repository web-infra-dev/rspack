/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  output: {
    chunkFilename: 'async.js',
  },
  optimization: {
    innerGraph: true,
    minimize: false,
    providedExports: true,
    sideEffects: true,
    usedExports: true,
  },
};
