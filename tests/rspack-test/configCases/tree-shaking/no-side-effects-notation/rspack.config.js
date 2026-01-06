module.exports = {
  mode: 'production',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false
  },
  experiments: {
    advancedTreeShaking: true,
  },
}