const path = require('path');

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: './src/index.js',
  externals: {
    fs: 'module fs',
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(__dirname, 'src'),
    },
  },
};
