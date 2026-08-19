const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  cache: { type: 'memory' },
  experiments: {
    newCache: true,
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
            options: { name: 'source-map-consumer' },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'source-map-producer' },
          },
          {
            loader: path.resolve(__dirname, 'marked-loader.js'),
            options: { name: 'marked' },
            parallel: { maxWorkers: 1 },
            cache: true,
          },
          {
            loader: path.resolve(__dirname, 'right-loader.js'),
            options: { name: 'right' },
            parallel: { maxWorkers: 1 },
            cache: true,
          },
        ],
      },
      {
        test: /module-[ab]\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'module-id' },
            cache: true,
          },
        ],
      },
    ],
  },
};
