/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        test: /\.js/,
        loader: './loader.mjs',
        issuerLayer: 'main',
        options: {},
      },
    ],
  },
};
