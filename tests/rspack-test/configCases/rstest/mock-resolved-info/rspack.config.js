const path = require('path');
const {
  experiments: { RstestPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = [
  // Entry 1: codegen target — emitted as `.mjs` and inspected by entry 2,
  // never run.
  {
    entry: './src/fixture.js',
    target: 'node',
    experiments: {
      outputModule: true,
    },
    output: {
      filename: 'mockResolvedInfo.mjs',
      module: true,
      chunkFormat: 'module',
    },
    externalsType: 'module-import',
    externals: {
      'node:os': 'node:os',
    },
    optimization: {
      concatenateModules: false,
      minimize: false,
      moduleIds: 'named',
      chunkIds: 'named',
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: true,
        hoistMockModule: true,
        importMetaPathName: true,
        manualMockRoot: path.resolve(__dirname, '__mocks__'),
      }),
    ],
  },
  // Entry 2: the test. Reads entry 1 output and asserts the codegen contract.
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
