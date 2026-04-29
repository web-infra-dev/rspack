const path = require('path');
const {
  experiments: { RstestPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = [
  {
    entry: './src/index.js',
    target: 'node',
    experiments: {
      outputModule: true,
    },
    output: {
      filename: 'dynamicImportOrigin.mjs',
      module: true,
      importFunctionName: 'import.meta.__rstest_dynamic_import__',
      chunkFormat: 'module',
    },
    module: {
      parser: {
        javascript: {
          importDynamic: false,
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
        importMetaPathName: true,
        manualMockRoot: path.resolve(__dirname, '__mocks__'),
        injectDynamicImportOrigin: true,
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
