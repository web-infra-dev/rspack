/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index\.js$/,
        use: [
          {
            loader: './import-loader.js',
            options: {},
            parallel: true,
          },
        ],
      },
    ],
  },
};
