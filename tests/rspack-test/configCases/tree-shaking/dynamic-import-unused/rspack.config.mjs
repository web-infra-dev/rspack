/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  output: {
    chunkFilename: 'chunk.js',
  },
  optimization: {
    minimize: true,
    providedExports: true,
    usedExports: true,
    sideEffects: true,
    innerGraph: true,
  },
};
