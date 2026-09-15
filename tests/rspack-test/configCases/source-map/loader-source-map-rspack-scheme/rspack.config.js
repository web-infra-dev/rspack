'use strict';

/** @type {import('@rspack/core').Configuration} */
const config = {
  devtool: 'source-map',
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /(?:module|https|data)\.js$/,
        use: [
          {
            loader: require.resolve('./loader.js'),
          },
        ],
      },
    ],
  },
};

module.exports = config;
