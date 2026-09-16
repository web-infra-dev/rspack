/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    // The stale (pre-apply) runtime resolves the newly added stylesheet, so
    // the css filename must be derivable from the chunk id alone.
    cssFilename: '[name].css',
    cssChunkFilename: '[name].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
