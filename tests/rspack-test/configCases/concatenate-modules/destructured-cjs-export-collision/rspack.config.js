/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    concatenateModules: true,
    mangleExports: 'deterministic',
    minimize: false,
    usedExports: true,
  },
};
