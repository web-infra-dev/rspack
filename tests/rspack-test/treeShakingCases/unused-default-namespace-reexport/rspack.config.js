/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    fasterModuleConcatenation: true,
  },
  optimization: {
    concatenateModules: true,
    sideEffects: true,
    minimize: false,
  },
};
