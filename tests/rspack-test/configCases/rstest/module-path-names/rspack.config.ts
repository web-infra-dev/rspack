import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack } from '@rspack/core';

const {
  experiments: { RstestPlugin },
} = rspack;

export default defineConfig([
  {
    entry: './src/index.js',
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
    output: {
      filename: 'modulePathName.js',
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: true,
        hoistMockModule: true,
        importMetaPathName: true,
        manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
      }),
    ],
  },
  {
    entry: './src/index.js',
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
    output: {
      filename: 'modulePathNameWithoutConcatenate.js',
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: true,
        hoistMockModule: true,
        importMetaPathName: true,
        manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
      }),
    ],
    optimization: {
      concatenateModules: false,
    },
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
