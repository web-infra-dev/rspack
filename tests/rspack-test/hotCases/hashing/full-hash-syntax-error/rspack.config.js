module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  node: {
    __dirname: false,
  },
  optimization: {
    emitOnErrors: false,
  },
};
