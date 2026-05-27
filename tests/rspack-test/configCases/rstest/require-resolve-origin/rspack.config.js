const path = require('path');
const {
  experiments: { RstestPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = [
  {
    entry: './src/index.js',
    target: 'node',
    output: {
      filename: 'requireResolveOrigin.js',
    },
    optimization: {
      concatenateModules: false,
      minimize: false,
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: false,
        hoistMockModule: false,
        importMetaPathName: false,
        manualMockRoot: path.resolve(__dirname, '__mocks__'),
        injectRequireResolveOrigin: true,
      }),
    ],
  },
  {
    entry: './src/index.js',
    target: 'node',
    output: {
      filename: 'requireResolveOriginFunctionName.js',
    },
    optimization: {
      concatenateModules: false,
      minimize: false,
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: false,
        hoistMockModule: false,
        importMetaPathName: false,
        manualMockRoot: path.resolve(__dirname, '__mocks__'),
        injectRequireResolveOrigin: {
          functionName: 'globalThis.__custom_require_resolve__',
        },
      }),
    ],
  },
  {
    entry: './src/index.js',
    target: 'node',
    output: {
      filename: 'requireResolveOriginMagicComments.js',
    },
    module: {
      parser: {
        javascript: {
          commonjsMagicComments: true,
        },
      },
    },
    optimization: {
      concatenateModules: false,
      minimize: false,
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: false,
        hoistMockModule: false,
        importMetaPathName: false,
        manualMockRoot: path.resolve(__dirname, '__mocks__'),
        injectRequireResolveOrigin: true,
      }),
    ],
  },
  {
    entry: {
      main: './index.js',
    },
    output: {
      filename: '[name].js',
    },
    externalsPresets: {
      node: true,
    },
  },
];
