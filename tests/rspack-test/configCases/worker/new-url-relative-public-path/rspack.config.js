/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'none',
  output: {
    module: true,
    filename: 'main.js',
    chunkFilename: '[name].bundle.js',
    publicPath: '/public/',
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
