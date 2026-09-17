/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    sideEffects: true,
    usedExports: false,
    innerGraph: true,
  },
  module: {
    rules: [
      {
        test: /re-exports\.js$/,
        sideEffects: false,
      },
    ],
  },
};
