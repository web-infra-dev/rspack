/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: {
      type: 'jsonp',
      name: ['not', 'a', 'string'],
    },
  },
};
