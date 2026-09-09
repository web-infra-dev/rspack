module.exports = {
  target: 'node',
  module: {
    parser: {
      javascript: {
        requireAlias: true,
        createRequire: true,
      },
    },
  },
};
