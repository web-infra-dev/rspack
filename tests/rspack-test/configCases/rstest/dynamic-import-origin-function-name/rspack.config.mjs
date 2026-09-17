import path from 'node:path';
import { rspack } from '@rspack/core';

const {
  experiments: { RstestPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default [
  {
    entry: './src/index.js',
    target: 'node',
    experiments: {
      outputModule: true,
    },
    output: {
      filename: 'dynamicImportOrigin.mjs',
      module: true,
      // Intentionally NOT setting `importFunctionName` — its default is
      // `'import'`, which the rewrite normalizes to "feature off". A
      // successful rewrite in this fixture therefore proves the
      // `{ functionName }` override flowed through the N-API conversion
      // and apply-time normalization.
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
        injectDynamicImportOrigin: {
          functionName: 'globalThis.__custom_import__',
        },
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
