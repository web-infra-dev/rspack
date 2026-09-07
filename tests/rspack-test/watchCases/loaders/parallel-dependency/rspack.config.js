const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  module: {
    rules: [
      {
        // The worker returns immediately after this uncached parallel loader.
        test: /worker-return\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'dependency-loader.js'),
            options: { dependency: 'return-dependency.txt' },
            parallel: { maxWorkers: 1 },
            cache: false,
          },
        ],
      },
      {
        // The worker yields after the uncached parallel loader because the next loader runs on the
        // main thread.
        test: /worker-yield\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'passthrough-loader.js'),
          },
          {
            loader: path.resolve(__dirname, 'dependency-loader.js'),
            options: { dependency: 'yield-dependency.txt' },
            parallel: { maxWorkers: 1 },
          },
        ],
      },
    ],
  },
};
