/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: 'deterministic',
    minimize: false,
    providedExports: true,
    usedExports: true,
  },
};
