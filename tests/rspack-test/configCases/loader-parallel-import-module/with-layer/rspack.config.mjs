/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        test: /\.js/,
        use: [
          {
            loader: './loader',
            options: {},
            parallel: true,
          },
        ],
        issuerLayer: 'main',
      },
    ],
  },
};
