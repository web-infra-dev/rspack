/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  cache: true,
  output: {
    chunkFilename: '[contenthash].js',
  },
};
