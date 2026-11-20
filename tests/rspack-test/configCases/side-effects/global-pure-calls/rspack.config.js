module.exports = [false, true].map((pureFunctions) => ({
  externals: {
    fs: 'node-commonjs fs',
  },
  experiments: { pureFunctions },
  optimization: {
    sideEffects: true,
    moduleIds: 'named',
    concatenateModules: false,
    minimize: false,
  },
}));
