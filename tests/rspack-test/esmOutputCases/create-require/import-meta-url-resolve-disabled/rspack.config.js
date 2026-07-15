module.exports = {
  module: {
    parser: {
      javascript: {
        requireResolve: false,
      },
    },
    rules: [
      {
        test: /preserve-import-meta\.js$/,
        parser: {
          importMeta: false,
        },
      },
    ],
  },
};
