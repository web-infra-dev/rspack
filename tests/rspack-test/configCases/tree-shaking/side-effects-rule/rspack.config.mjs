/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /package/,
        sideEffects: false,
      },
    ],
  },

  optimization: {
    sideEffects: true,
  },
  externalsPresets: {
    node: true,
  },
};
