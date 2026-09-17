/** @type {import("@rspack/core").Configuration} */
export default {
  entry() {
    return {
      a: { import: './a' },
      b: { import: ['./b'] },
    };
  },
  output: {
    filename: '[name].js',
  },
};
