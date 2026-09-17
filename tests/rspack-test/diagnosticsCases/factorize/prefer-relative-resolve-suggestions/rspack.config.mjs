/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/esm',
      },
    ],
  },
};
