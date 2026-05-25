/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    // Avoid the default export of module/b.js is inlined into module/index.js,
    // which causes the side effect of module/b.js not executed.
    inlineExports: false,
  },
};
