/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /index\.js/,
        use: [
          {
            loader: './test-loader.js',
          },
        ],
      },
    ],
  },
};
