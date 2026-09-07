module.exports = {
  mode: 'development',
  module: {
    parser: {
      javascript: {
        worker: {
          url: 'new-url-relative',
        },
      },
    },
  },
  optimization: {
    runtimeChunk: false,
  },
};
