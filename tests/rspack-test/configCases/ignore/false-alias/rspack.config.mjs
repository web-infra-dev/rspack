/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './test.js',
  resolve: {
    alias: {
      'ignored-module': false,
      './ignored-module': false,
    },
  },
};
