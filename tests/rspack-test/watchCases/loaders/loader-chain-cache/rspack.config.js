const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  experiments: {
    newCache: true,
  },
  cache: {
    type: 'memory',
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
            parallel: { maxWorkers: 1 },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'right' },
            parallel: { maxWorkers: 1 },
            cache: true,
          },
        ],
      },
    ],
  },
};
