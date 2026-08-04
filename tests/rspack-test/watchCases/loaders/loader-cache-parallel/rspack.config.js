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
            options: {},
            parallel: { maxWorkers: 1 },
            cache: true,
          },
        ],
      },
    ],
  },
};
