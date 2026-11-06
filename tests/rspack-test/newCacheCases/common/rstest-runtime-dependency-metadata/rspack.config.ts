import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

let compilerIndex = 0;
const loaderOptions: { builtModules: string[] } = { builtModules: [] };

export default defineConfig({
  context: import.meta.dirname,
  cache: { type: 'persistent' },
  experiments: {
    newCache: {
      codeGeneration: false,
      devtool: false,
      loader: false,
      minimize: false,
      module: true,
    },
  },
  module: {
    parser: { javascript: { importDynamic: false } },
    rules: [
      {
        test: /(?:stable|changed)\.js$/,
        loader: './loader.mjs',
        options: loaderOptions,
      },
    ],
  },
  optimization: { concatenateModules: false },
  plugins: [
    new rspack.experiments.RstestPlugin({
      injectModulePathName: false,
      hoistMockModule: false,
      importMetaPathName: false,
      manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
      injectDynamicImportOrigin: { functionName: '__rstest_import__' },
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CheckRuntimeDependencyCache', (stats) => {
          const modules = Array.from(stats.compilation.modules);
          const metadata = (name: string) =>
            modules.find((module) =>
              module
                .nameForCondition()
                ?.replaceAll('\\', '/')
                .endsWith(`/${name}.js`),
            )?.buildInfo.rstestRuntimeDependencies;

          expect(metadata('stable')).toEqual({
            version: 1,
            calls: [
              {
                kind: 'import',
                origin: 'source',
                parts: ['./stable/', null, '.mjs'],
              },
            ],
          });
          expect(metadata('changed')).toEqual({
            version: 1,
            calls:
              compilerIndex < 2
                ? [
                    {
                      kind: 'import',
                      origin: 'source',
                      parts: ['./changed/', null, '.mjs'],
                    },
                  ]
                : [],
          });
          expect(
            loaderOptions.builtModules
              .map((name) => path.basename(name))
              .sort(),
          ).toEqual(
            compilerIndex === 0
              ? ['changed.js', 'stable.js']
              : compilerIndex === 1
                ? []
                : ['changed.js'],
          );
          loaderOptions.builtModules = [];
          compilerIndex++;
        });
      },
    }),
  ],
});
