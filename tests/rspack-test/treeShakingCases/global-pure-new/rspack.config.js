/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    builtinPureGlobals: true,
  },
  optimization: {
    sideEffects: true,
  },
};
