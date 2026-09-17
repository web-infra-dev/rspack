/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  output: {
    library: { type: 'window', name: ['a', 'b'] },
  },
};
