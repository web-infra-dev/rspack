/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    '@rspack/test-tools': 'commonjs @rspack/test-tools',
  },
  optimization: {
    splitChunks: false,
    sideEffects: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
};
