/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index\.js/,
        loader: './loader.mjs',
        options: {},
      },
    ],
  },
};
