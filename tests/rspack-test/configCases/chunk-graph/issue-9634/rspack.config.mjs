/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    b: './entry-b',
    a: './entry-a',
  },
  output: {
    filename: '[name].js',
  },
};
