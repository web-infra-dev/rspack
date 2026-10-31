import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { ConcatenatedModule, rspack, type Compiler } from '@rspack/core';

const parser = {
  importDynamic: false,
  requireDynamic: false,
  requireAsExpression: false,
  requireResolve: false,
  commonjsMagicComments: true,
  createRequire: true,
};

const configurations = [
  'global',
  'rule',
  'concatenated',
  'bundled-require',
].map((variant) =>
  defineConfig({
    entry:
      variant === 'concatenated'
        ? './src/entry.js'
        : variant === 'bundled-require'
          ? './src/bundled.cjs'
          : './src/index.js',
    target: 'node',
    output: {
      filename: `${variant}.js`,
    },
    module: {
      parser: {
        javascript: variant === 'rule' ? {} : parser,
      },
      rules:
        variant === 'rule'
          ? [{ test: /index\.js$/, type: 'javascript/auto', parser }]
          : variant === 'bundled-require'
            ? [{ test: /bundled\.cjs$/, parser: { requireDynamic: true } }]
            : [],
    },
    optimization: {
      concatenateModules: variant === 'concatenated',
      minimize: false,
    },
    plugins: [
      new rspack.experiments.RstestPlugin({
        injectModulePathName: false,
        hoistMockModule: false,
        importMetaPathName: false,
        manualMockRoot: path.resolve(import.meta.dirname, '__mocks__'),
        injectDynamicImportOrigin: { functionName: '__rstest_import__' },
        injectRequireResolveOrigin: { functionName: '__rstest_resolve__' },
      }),
      {
        apply(compiler: Compiler) {
          compiler.hooks.thisCompilation.tap(
            'RuntimeDependencyMetadata',
            (compilation) => {
              compilation.hooks.processAssets.tap(
                'RuntimeDependencyMetadata',
                () => {
                  const modules = Array.from(compilation.modules);
                  const originals = modules.flatMap((module) =>
                    module instanceof ConcatenatedModule
                      ? [module, ...module.modules]
                      : [module],
                  );
                  const entry = originals.find((module) =>
                    module
                      .nameForCondition()
                      ?.replaceAll('\\', '/')
                      .endsWith(
                        variant === 'bundled-require'
                          ? '/src/bundled.cjs'
                          : '/src/index.js',
                      ),
                  );
                  compilation.emitAsset(
                    `${variant}.json`,
                    new rspack.sources.RawSource(
                      JSON.stringify({
                        metadata: entry?.buildInfo.rstestRuntimeDependencies,
                        modules: Array.from(
                          new Set(
                            originals
                              .map((module) => module.nameForCondition())
                              .filter(Boolean),
                          ),
                        ),
                        concatenated: modules.some(
                          (module) =>
                            module instanceof ConcatenatedModule &&
                            module.modules.length > 1,
                        ),
                      }),
                    ),
                  );
                },
              );
            },
          );
        },
      },
    ],
  }),
);

export default defineConfig([
  ...configurations,
  {
    entry: './index.js',
    output: { filename: 'main.js' },
    externalsPresets: { node: true },
  },
]);
