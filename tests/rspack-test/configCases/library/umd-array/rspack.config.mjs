/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'umd',
      root: ['test', 'library'],
      amd: 'test-library',
      commonjs: 'test-library',
    },
  },
};
