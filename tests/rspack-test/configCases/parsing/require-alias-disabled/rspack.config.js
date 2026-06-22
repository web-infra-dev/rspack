module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  module: {
    parser: {
      javascript: {},
    },
  },
};
