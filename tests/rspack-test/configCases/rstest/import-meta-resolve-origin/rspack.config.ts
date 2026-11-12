import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  [true, false].map((defaultName) => ({
    target: 'node',
    module: {
      parser: { javascript: { importMetaResolve: !defaultName } },
      rules: [
        {
          test: defaultName
            ? /bundler-resolve\.js$/
            : /(?:index|resolver)\.js$/,
          parser: { importMetaResolve: defaultName },
        },
      ],
    },
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
