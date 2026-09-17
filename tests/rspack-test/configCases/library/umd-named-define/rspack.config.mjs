/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'umd',
      root: 'testLibrary[name]',
      amd: 'test-library-[name]',
      commonjs: 'test-library-[name]',
      umdNamedDefine: true,
    },
  },
  externals: 'module',
};
