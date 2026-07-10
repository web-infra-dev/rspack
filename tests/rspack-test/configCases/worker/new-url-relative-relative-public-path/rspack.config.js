/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'none',
  output: {
    module: true,
    filename: 'js/main.js',
    chunkFilename: '[name].bundle.js',
    publicPath: 'assets/',
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
