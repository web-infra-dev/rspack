/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  optimization: {
    innerGraph: true,
    minimize: false,
    sideEffects: true,
    usedExports: true,
  },
  experiments: {
    pureFunctions: true,
  },
};
