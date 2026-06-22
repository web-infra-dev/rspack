/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
};
