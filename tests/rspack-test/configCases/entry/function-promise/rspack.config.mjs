/** @type {import("@rspack/core").Configuration} */
export default {
  entry() {
    return Promise.resolve({
      a: './a',
      b: ['./b'],
    });
  },
  output: {
    filename: '[name].js',
  },
};
