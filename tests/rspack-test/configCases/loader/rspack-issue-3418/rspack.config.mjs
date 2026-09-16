/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js$/,
        loader: './loader.js',
      },
    ],
  },
};
