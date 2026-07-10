module.exports = {
  module: {
    parser: {
      javascript: {
        createRequire: 'createRequire from node:module',
        requireResolve: false,
      },
    },
  },
};
