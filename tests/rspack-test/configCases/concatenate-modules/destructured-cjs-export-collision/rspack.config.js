/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  experiments: {
    fasterModuleConcatenation: true,
  },
  optimization: {
    concatenateModules: true,
    mangleExports: 'deterministic',
    minimize: false,
    usedExports: true,
  },
};
