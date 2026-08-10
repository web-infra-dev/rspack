const path = require('path');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
  },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          path.resolve(__dirname, 'left-loader.js'),
          {
            loader: path.resolve(__dirname, 'loader.js'),
            cache: true,
          },
        ],
      },
    ],
  },
};
