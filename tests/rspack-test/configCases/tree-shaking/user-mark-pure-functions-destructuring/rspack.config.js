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
        test: /dep\.js$/,
        parser: {
          pureFunctions: ['fromObject', 'fromArray'],
        },
      },
    ],
  },
};
