/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    inlineExports: true,
    moduleIds: 'named',
    usedExports: true,
  },
};
