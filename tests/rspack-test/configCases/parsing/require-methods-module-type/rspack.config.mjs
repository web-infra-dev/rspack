/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /cjs\.js$/,
        type: 'javascript/dynamic',
      },
      {
        test: /esm\.js$/,
        type: 'javascript/esm',
      },
    ],
  },
};
