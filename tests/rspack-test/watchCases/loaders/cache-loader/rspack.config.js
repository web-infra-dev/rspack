const path = require('node:path');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  cache: false,
  module: {
    rules: [
      {
        test: /[/\\]value\.js$/,
        use: [
          {
            loader: path.resolve(__dirname, 'passthrough-loader.js'),
            cache: true,
          },
          path.resolve(__dirname, 'count-loader.js'),
        ],
      },
      {
        test: /builtin-value\.js$/,
        use: [
          {
            loader: 'builtin:test-passthrough-loader',
            cache: true,
          },
          path.resolve(__dirname, 'builtin-count-loader.js'),
        ],
      },
    ],
  },
};
