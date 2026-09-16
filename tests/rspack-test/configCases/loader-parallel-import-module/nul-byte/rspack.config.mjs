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
            loader: './convert-loader.js',
            options: {},
            parallel: true,
          },
        ],
      },
    ],
  },
};
