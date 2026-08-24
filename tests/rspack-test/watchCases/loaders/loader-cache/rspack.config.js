const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  cache: { type: 'memory' },
  experiments: {
    newCache: {
      codeGeneration: false,
      loader: true,
      minimize: false,
      // This case asserts that an uncached loader reruns on every step, which
      // the module build cache would skip for a rewrite with equal content.
      module: false,
    },
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
        test: /bom\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'bom-consumer' },
          },
          {
            loader: path.resolve(__dirname, 'loader.js'),
            options: { name: 'bom-producer' },
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
