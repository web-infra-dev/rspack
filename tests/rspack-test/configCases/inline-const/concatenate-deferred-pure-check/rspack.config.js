/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    concatenateModules: true,
    inlineExports: true,
    minimize: false,
    providedExports: true,
    sideEffects: true,
    usedExports: true,
  },
};
