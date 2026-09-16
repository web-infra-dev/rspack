/** @type {import("@rspack/core").Configuration} */
export default {
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
