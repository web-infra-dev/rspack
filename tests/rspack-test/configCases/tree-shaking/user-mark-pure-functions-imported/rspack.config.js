module.exports = {
  mode: 'production',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false,
  },
  module: {
    rules: [
      {
        test: /consumer\.js$/,
        parser: {
          pureFunctions: ['libFn'],
        },
      },
    ],
  },
};
