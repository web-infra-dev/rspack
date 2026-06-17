module.exports = {
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false,
  },
  module: {
    rules: [
      {
        test: /decl\.js$/,
        parser: {
          pureFunctions: ['a', 'b'],
        },
      },
    ],
  },
};
