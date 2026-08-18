/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    concatenateModules: {
      commonjs: false,
    },
    inlineExports: false,
  },
};
