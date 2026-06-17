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
        test: /decl\.js/,
        parser: {
          pureFunctions: ['notExistFunction'],
        },
      },
    ],
  },
};
