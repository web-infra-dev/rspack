const path = require('path');

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
  extends: [require.resolve('./base.rspack.config.js')],
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
  },
};
