/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'none',
  output: {
    filename: ({ chunk }) =>
      chunk.name === 'main' ? 'js/main.js' : 'runtime.bundle.js',
    chunkFilename: 'worker.bundle.js',
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
