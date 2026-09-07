/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  entry: {
    unused: './unused-entry.js',
    used: './used-entry.js',
    second: './second-used-entry.js',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    innerGraph: true,
    minimize: false,
    providedExports: true,
    sideEffects: true,
    usedExports: true,
  },
};
