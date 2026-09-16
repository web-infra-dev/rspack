/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'umd' },
  },
  externals: {
    external: 'external',
  },
};
