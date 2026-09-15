const path = require('path');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: './src/index.js',
  externals: {
    'node:events': 'module node:events',
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(__dirname, 'src'),
    },
  },
  optimization: {
    mangleExports: 'size',
    minimize: false,
  },
};
