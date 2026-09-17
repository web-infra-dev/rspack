/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'umd',
      root: 'testLibrary[name]',
      amd: 'test-library',
      commonjs: 'test-library-[name]',
    },
  },
  externals: 'module',
};
