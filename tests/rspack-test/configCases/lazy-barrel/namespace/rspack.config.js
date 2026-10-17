module.exports = {
  experiments: { lazyBarrel: true },
  optimization: {
    usedExports: false,
    sideEffects: 'flag',
    concatenateModules: false,
  },
};
