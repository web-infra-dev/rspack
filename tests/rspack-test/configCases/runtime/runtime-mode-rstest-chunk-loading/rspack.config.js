const path = require('path');
const {
  experiments: { RstestPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  experiments: {
    RstestPlugin,
    runtimeMode: 'rspack',
  },
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  plugins: [
    new RstestPlugin({
      injectModulePathName: false,
      importMetaPathName: false,
      hoistMockModule: false,
      manualMockRoot: path.resolve(__dirname, '__mocks__'),
    }),
  ],
};
