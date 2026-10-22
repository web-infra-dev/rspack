import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

const {
  experiments: { RstestPlugin },
} = rspack;

export default defineConfig([
  {
    entry: './src/fixture.js',
    target: 'node',
    mode: 'production',
    output: {
      filename: 'usedExports.mjs',
      module: true,
      chunkFormat: 'module',
    },
    externalsType: 'module-import',
    externals: {
      '@rstest/core': '@rstest/core',
    },
    optimization: {
      usedExports: true,
      providedExports: true,
      sideEffects: true,
      concatenateModules: false,
      minimize: false,
      moduleIds: 'named',
      chunkIds: 'named',
    },
    plugins: [
      new RstestPlugin({
        injectModulePathName: true,
        injectImportMetaRstestOrigin: true,
        importMetaPathName: true,
        hoistMockModule: true,
        manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
        globals: true,
        updateImportMockAPI: true,
        updateRequireMockAPI: true,
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
