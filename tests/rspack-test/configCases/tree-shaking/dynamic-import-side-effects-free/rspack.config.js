/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    minimize: false,
    providedExports: true,
    usedExports: true,
    sideEffects: true,
    innerGraph: true,
  },
};
