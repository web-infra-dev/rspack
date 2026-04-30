const path = require('path');

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
  entry: './src/index1.js',
  name: 'base_config',
  mode: 'development',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle1.js',
  },
};
