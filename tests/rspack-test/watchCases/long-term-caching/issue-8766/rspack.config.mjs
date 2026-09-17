/** @type {import("@rspack/core").Configuration} */
export default {
  incremental: false,
  mode: 'production',
  output: {
    chunkFilename: '[contenthash].js',
  },
};
