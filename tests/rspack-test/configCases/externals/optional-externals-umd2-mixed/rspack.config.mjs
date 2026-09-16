/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'umd2' },
  },
  externals: {
    external: 'external',
    external2: 'fs',
  },
};
