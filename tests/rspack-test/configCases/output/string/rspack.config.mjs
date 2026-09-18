/** @type {import("@rspack/core").Configuration} */
export default {
  entry() {
    return {
      a: './a',
    };
  },
  output: {
    filename: '[name].js',
  },
};
