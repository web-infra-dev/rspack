module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  module: {
    parser: {
      javascript: {
        createRequire: false,
      },
    },
  },
};
