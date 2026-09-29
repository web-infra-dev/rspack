import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

const {
  experiments: { RstestPlugin },
} = rspack;

export default defineConfig([
  {
    entry: './src/index.js',
    target: 'node',
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
        manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
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
]);
