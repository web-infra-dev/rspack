/** @type {import("@rspack/core").Configuration} */
export default {
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
