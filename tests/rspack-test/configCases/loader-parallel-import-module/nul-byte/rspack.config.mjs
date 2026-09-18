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
        use: [
          {
            loader: './convert-loader.mjs',
            options: {},
            parallel: true,
          },
        ],
      },
    ],
  },
};
