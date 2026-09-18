/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    'main-one': {
      import: ['./index-one.js'],
    },
    'main-two': {
      import: ['./index-two.js'],
    },
  },
};
