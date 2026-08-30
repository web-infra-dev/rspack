module.exports = {
  experiments: {
    pureFunctions: true,
  },
  optimization: {
    sideEffects: true,
    providedExports: true,
    usedExports: true,
    innerGraph: true,
  },
};
