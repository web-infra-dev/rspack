const path = require('path');

/**
 * @type {import('@rspack/core').RspackOptions}
 */
// Should apply correctly even if the config is an array
// See https://github.com/web-infra-dev/rspack/issues/10745
module.exports = [
  {
    extends: [
      path.resolve(__dirname, 'base.rspack.config.js'),
      path.resolve(__dirname, 'dev.rspack.config.js'),
    ],
    entry: './src/index.js',
    output: {
      path: path.resolve(__dirname, 'dist'),
    },
  },
];
