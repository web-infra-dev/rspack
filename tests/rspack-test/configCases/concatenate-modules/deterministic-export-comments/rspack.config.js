/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    concatenateModules: true,
    inlineExports: true,
    mangleExports: false,
    minimize: false,
    providedExports: true,
    usedExports: true,
  },
};
