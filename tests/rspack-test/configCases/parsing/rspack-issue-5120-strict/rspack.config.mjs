/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
    fail: './fail.js',
  },
  output: {
    filename: '[name].js',
  },
};
