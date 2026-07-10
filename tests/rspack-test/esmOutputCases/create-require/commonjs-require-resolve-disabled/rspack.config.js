module.exports = {
  output: {
    filename: 'main.js',
    library: {
      type: 'commonjs',
    },
    module: false,
  },
  optimization: {
    runtimeChunk: false,
  },
  module: {
    parser: {
      javascript: {
        createRequire: true,
        requireResolve: false,
      },
    },
  },
};
