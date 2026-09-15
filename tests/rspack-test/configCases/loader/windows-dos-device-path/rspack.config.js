const path = require('node:path');

const context = `\\\\?\\${path.resolve(__dirname)}`;

/** @type {import('@rspack/core').RspackOptions} */
module.exports = {
  context,
  entry: './index.js',
  resolve: {
    symlinks: false,
  },
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: [`${path.join(context, 'loader.js')}?loader-query`],
      },
    ],
  },
};
