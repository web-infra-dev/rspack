import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  [true, false].map((defaultName) => ({
    target: 'node',
    optimization: { minimize: false, concatenateModules: true },
    plugins: [
      new rspack.experiments.RstestPlugin({
        injectModulePathName: false,
        importMetaPathName: false,
        hoistMockModule: false,
        manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
        injectImportMetaResolveOrigin: defaultName
          ? true
          : {
              functionName: 'globalThis.__custom_import_meta_resolve__',
            },
      }),
    ],
  })),
);
