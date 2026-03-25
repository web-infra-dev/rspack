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
  module: {
    rules: [
      {
        test: /dep-named\.js$/,
        parser: {
          sideEffectsFree: ['pureNamed']
        }
      },
      {
        test: /dep-default\.js$/,
        parser: {
          sideEffectsFree: ['default']
        }
      }
    ]
  },
};
