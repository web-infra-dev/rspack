/** @type {import("@rspack/core").Configuration} */
export default {
  entry() {
    return {
      a: './a',
      b: ['./b'],
    };
  },
  output: {
    filename: '[name].js',
  },
};
