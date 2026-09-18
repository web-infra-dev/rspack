/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    alias: {
      'multi-alias': ['./b', './a'],
    },
  },
};
