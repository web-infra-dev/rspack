/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'none',
  output: {
    filename: 'main.js',
    chunkFilename: '[name].bundle.js',
    library: {
      type: 'modern-module',
    },
  },
  module: {
    parser: {
      javascript: {
        worker: {
          url: 'new-url-relative',
        },
      },
    },
  },
};
