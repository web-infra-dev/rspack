/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    a1: './a',
    b1: {
      runtime: 'a1',
      import: './b',
    },
  },
};
