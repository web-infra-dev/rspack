/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'system',
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};
