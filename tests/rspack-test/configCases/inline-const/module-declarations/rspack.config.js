/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    concatenateModules: false,
    inlineExports: true,
    moduleIds: 'named',
    sideEffects: true,
    usedExports: true,
  },
};
