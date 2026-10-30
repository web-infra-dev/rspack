import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { rspack, type Compiler } from '@rspack/core';

export default defineConfig([
  {
    entry: './src/entry.js',
    target: 'node',
    output: { filename: 'capability.js', library: { type: 'commonjs2' } },
    module: {
      parser: {
        javascript: {
          importDynamic: false,
          requireDynamic: false,
          requireAsExpression: false,
          requireResolve: false,
        },
      },
      rules: [{ test: /folded\.cjs$/, parser: { requireDynamic: true } }],
    },
    optimization: { concatenateModules: false, minimize: false },
    plugins: [
      new rspack.DefinePlugin({
        __FIXTURE_PARENT__: JSON.stringify(
          path.join(import.meta.dirname, 'src/entry.js'),
        ),
      }),
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
            'ObserveNativeCapability',
            (compilation) => {
              compilation.hooks.processAssets.tap(
                'ObserveNativeCapability',
                () => {
                  compilation.emitAsset(
                    'capability.json',
                    new rspack.sources.RawSource(
                      JSON.stringify({
                        createRequire:
                          compiler.options.module.parser?.javascript
                            ?.createRequire,
                        modules: Array.from(compilation.modules).flatMap(
                          (module) => {
                            const resource = module.nameForCondition();
                            return resource
                              ? [
                                  {
                                    resource: path.basename(resource),
                                    metadata:
                                      module.buildInfo
                                        .rstestRuntimeDependencies,
                                  },
                                ]
                              : [];
                          },
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
  },
  {
    entry: './index.js',
    target: 'node',
    output: { filename: 'main.js' },
    externalsPresets: { node: true },
    module: {
      parser: {
        javascript: { requireDynamic: false, requireAsExpression: false },
      },
    },
  },
]);
