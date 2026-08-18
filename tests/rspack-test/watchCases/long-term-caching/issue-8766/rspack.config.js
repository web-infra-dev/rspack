/** @type {import("@rspack/core").Configuration} */
module.exports = {
  incremental: false,
  mode: 'production',
  output: {
    chunkFilename: '[contenthash].js',
  },
};
