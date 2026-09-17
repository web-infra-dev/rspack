/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'umd' },
  },
  externals: {
    external: {
      root: ['a', 'b'],
      commonjs: 'a',
      commonjs2: 'a',
      amd: 'a',
    },
  },
};
