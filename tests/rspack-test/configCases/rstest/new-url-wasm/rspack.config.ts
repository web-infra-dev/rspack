import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

const {
  experiments: { RstestPlugin },
} = rspack;

export default defineConfig([
  {
    entry: './index.js',
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
    output: {
      filename: 'bundle.js',
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: true,
        hoistMockModule: true,
        importMetaPathName: true,
        manualMockRoot: import.meta.dirname,
        preserveNewUrl: ['.wasm'],
      }),
    ],
  },
  {
    entry: {
      main: './test.js',
    },
    output: {
      filename: '[name].js',
    },
    externalsPresets: {
      node: true,
    },
  },
]);
