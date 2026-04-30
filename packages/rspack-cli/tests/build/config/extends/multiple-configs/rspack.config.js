const path = require('path');

/**
 * @type {import('@rspack/core').MultiRspackOptions}
 */
module.exports = [
  {
    name: 'derived_config1',
    extends: path.resolve(__dirname, 'base.rspack.config.js'),
    entry: './src/index1.js',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'bundle1.js',
    },
  },
  {
    name: 'derived_config2',
    extends: path.resolve(__dirname, 'base.rspack.config.js'),
    entry: './src/index2.js',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'bundle2.js',
    },
  },
];
