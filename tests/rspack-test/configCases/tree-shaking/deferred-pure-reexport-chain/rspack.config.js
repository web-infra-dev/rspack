/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'node',
  optimization: {
    sideEffects: true,
    innerGraph: true,
    usedExports: true,
    minimize: false,
    concatenateModules: false,
  },
  experiments: {
    pureFunctions: true,
  },
};
