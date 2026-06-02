/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  optimization: {
    inlineExports: true,
    moduleIds: 'named',
  },
};
