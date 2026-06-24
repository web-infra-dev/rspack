const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  output: {
    assetModuleFilename: '[hash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  experiments: {
    buildHttp: {
      allowedUris: ['https://'],
      lockfileLocation: path.resolve(__dirname, './lock-files/lock.json'),
      cacheLocation: path.resolve(__dirname, './lock-files/test'),
    },
  },
  externalsPresets: {},
};
