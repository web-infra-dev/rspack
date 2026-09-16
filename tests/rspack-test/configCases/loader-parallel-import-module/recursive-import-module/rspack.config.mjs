/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    publicPath: '/public/',
  },
  entry: './entry.js',
  module: {
    rules: [
      {
        test: /\.js/,
        use: [
          {
            loader: './loader.js',
            options: {},
            parallel: true,
          },
        ],
      },
    ],
  },
};
