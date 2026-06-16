/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  module: {
    rules: [
      {
        test: /file\.js$/,
        loader: './loader.js',
      },
    ],
  },
  cache: {
    type: 'persistent',
  },
};
