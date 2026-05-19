/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
    usedExports: false,
  },
};
