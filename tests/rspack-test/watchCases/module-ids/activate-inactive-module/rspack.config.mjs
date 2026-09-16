/** @type {import('@rspack/core').Configuration} */
export default {
  module: {
    rules: [
      {
        test: /a\.js/,
        sideEffects: false,
      },
    ],
  },
};
