/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    bundle0: './a',
    other: './b',
  },
  output: {
    filename: '[name].js',
  },
};
