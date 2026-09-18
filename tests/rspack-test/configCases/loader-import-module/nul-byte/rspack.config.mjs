/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    publicPath: '/public/',
  },
  entry: './index.js',
  module: {
    rules: [
      {
        test: /a.js/,
        loader: './convert-loader.mjs',
      },
    ],
  },
};
