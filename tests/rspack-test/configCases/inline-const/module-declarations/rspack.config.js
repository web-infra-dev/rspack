/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    concatenateModules: false,
    inlineExports: true,
    moduleIds: 'named',
    sideEffects: true,
    usedExports: true,
  },
};
