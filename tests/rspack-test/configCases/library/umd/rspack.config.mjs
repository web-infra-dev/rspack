/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'umd',
      root: 'testLibrary',
      amd: 'test-library',
      commonjs: 'test-library',
    },
  },
};
