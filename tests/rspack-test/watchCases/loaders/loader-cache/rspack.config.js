const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    loaderCache: true,
  },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'left' },
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'marked' },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'right' },
            cache: true,
          },
        ],
      },
    ],
  },
};
