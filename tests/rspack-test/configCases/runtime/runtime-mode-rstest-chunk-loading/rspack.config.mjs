import path from 'node:path';
import { rspack } from '@rspack/core';

const {
  experiments: { RstestPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default {
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
      manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
    }),
  ],
};
