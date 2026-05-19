const path = require('path');

/**
 * @type {import('@rspack/core').MultiRspackOptions}
 */
module.exports = [
  {
    extends: path.resolve(__dirname, 'base.rspack.config.js'),
  },
  {
    name: 'derived_config2',
    mode: 'development',
    entry: './src/index2.js',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'bundle2.js',
    },
  },
];
