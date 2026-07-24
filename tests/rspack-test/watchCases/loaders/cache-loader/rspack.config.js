const path = require('node:path');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'development',
  cache: false,
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          'builtin:cache-loader',
          path.resolve(__dirname, 'count-loader.js'),
        ],
      },
    ],
  },
};
