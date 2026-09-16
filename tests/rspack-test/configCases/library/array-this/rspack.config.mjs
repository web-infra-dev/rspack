/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  output: {
    library: { type: 'this', name: ['a', 'b'] },
    environment: {
      arrowFunction: false,
    },
  },
};
