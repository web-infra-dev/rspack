/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  output: {
    library: {
      type: 'commonjs',
    },
  },
  optimization: {
    minimize: false,
  },
};
