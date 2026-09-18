/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
};
