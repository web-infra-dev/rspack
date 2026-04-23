const path = require('path');
/**@type {import('@rspack/core').Configuration} */
module.exports = {
  target: 'web',
  entry: './src/index.js',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(__dirname, 'src'),
    },
  },
};
