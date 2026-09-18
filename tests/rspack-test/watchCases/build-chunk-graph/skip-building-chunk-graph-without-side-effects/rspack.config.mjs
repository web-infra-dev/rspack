/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    sideEffects: true,
    usedExports: false,
    innerGraph: true,
  },
  incremental: {
    buildChunkGraph: true,
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        sideEffects: false,
      },
    ],
  },
};
