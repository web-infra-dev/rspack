/**@type {import("@rspack/core").Configuration}*/
export default {
  module: {
    rules: [
      {
        test: /side-effect\.js/,
        sideEffects: false,
      },
    ],
  },
  optimization: {
    sideEffects: false,
  },
};
