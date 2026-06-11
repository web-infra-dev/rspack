module.exports = {
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
  },
  module: {
    rules: [
      {
        test: /decl\.js$/,
        parser: {
          pureFunctions: ['pureFn', 'a', 'b'],
        },
      },
      {
        test: /member-decl\.js$/,
        parser: {
          pureFunctions: ['memberPure'],
        },
      },
      {
        test: /unsafe-decl\.js$/,
        parser: {
          pureFunctions: ['unsafePureFn'],
        },
      },
    ],
  },
};
