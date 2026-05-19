/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
