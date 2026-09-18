/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index\.js/,
        use: [
          { loader: './import-loader.mjs', options: {}, parallel: true },
          { loader: './import-loader-2.mjs', options: {}, parallel: true },
        ],
      },
    ],
  },
};
