/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index\.js/,
        use: ['./import-loader.js', './import-loader-2.js'],
      },
    ],
  },
};
