/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    inlineExports: true,
    moduleIds: 'named',
    usedExports: true,
  },
};
