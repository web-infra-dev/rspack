/** @type {import("@rspack/core").Configuration} */
module.exports = {
  optimization: {
    // Avoid the default export of root.js is inlined into external.js,
    // which causes the side effect of root.js not executed.
    inlineExports: false,
  },
};
