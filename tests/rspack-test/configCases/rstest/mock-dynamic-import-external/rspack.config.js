const path = require('path');
const {
  experiments: { RstestPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = [
  // Entry 1: the codegen target. Built with rstest's real shape (outputModule +
  // `module-import` externals, the type that forks `external module` vs
  // `external import`). Emitted as `.mjs` and inspected by entry 2 — never run.
  {
    entry: './src/fixture.js',
    target: 'node',
    experiments: {
      outputModule: true,
    },
    output: {
      filename: 'mockDynamicImport.mjs',
      module: true,
      chunkFormat: 'module',
    },
    externalsType: 'module-import',
    externals: {
      'node:child_process': 'node:child_process',
      'node:child_process?weird': 'node:child_process',
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
  // Entry 2: the test. Reads entry 1's output and asserts the codegen contract.
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
