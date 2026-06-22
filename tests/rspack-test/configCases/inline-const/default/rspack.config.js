/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    inlineExports: true,
  },
};
