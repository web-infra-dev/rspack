import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default [false, true].map((esm, index) =>
  defineConfig({
    mode: 'development',
    target: 'node',
    entry: './index.js',
    devtool: false,
    experiments: { runtimeMode: 'rspack' },
    output: {
      filename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      chunkFilename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      publicPath: '/initial/',
      chunkLoading: esm ? 'import' : 'require',
      ...(esm ? { module: true, library: { type: 'modern-module' } } : {}),
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      runtimeChunk: 'single',
    },
    plugins: [
      new rspack.experiments.RstestPlugin({
        injectModulePathName: false,
        importMetaPathName: false,
        hoistMockModule: false,
        manualMockRoot: import.meta.dirname,
      }),
      definePlugin((compiler) => {
        new rspack.DefinePlugin({
          RUNTIME_FILENAME: JSON.stringify(
            `runtime${index}.${esm ? 'mjs' : 'js'}`,
          ),
        }).apply(compiler);
        compiler.hooks.thisCompilation.tap(
          'RuntimeSourceUpdates',
          (compilation) => {
            let generated = 0;
            let modifiedChunkLoading = false;
            const chunkLoadingConstructor = esm
              ? 'EsmChunkLoadingRuntimeModule'
              : 'RequireChunkLoadingRuntimeModule';
            class CustomRuntime extends rspack.RuntimeModule {
              constructor() {
                super('source-updates', rspack.RuntimeModule.STAGE_TRIGGER);
              }
              generate() {
                generated++;
                return '__webpack_require__.j = "removed";';
              }
            }
            compilation.hooks.additionalTreeRuntimeRequirements.tap(
              'RuntimeSourceUpdates',
              (chunk, requirements) => {
                compilation.addRuntimeModule(chunk, new CustomRuntime());
                requirements.add(rspack.RuntimeGlobals.ensureChunkHandlers);
                if (generated !== 1) {
                  throw new Error(
                    'Custom source must be generated when it is added',
                  );
                }
              },
            );
            compilation.hooks.runtimeModule.tap(
              'FirstSourceUpdate',
              (module) => {
                if (!module.source) return;
                if (module.name === 'source-updates') {
                  if (generated !== 1) {
                    throw new Error(
                      'The hook must reuse the source analyzed during addRuntimeModule',
                    );
                  }
                  module.source.source = '__webpack_require__.nc = "removed";';
                }
                if (module.constructor.name === chunkLoadingConstructor) {
                  modifiedChunkLoading = true;
                  module.source.source = `${module.generate()}\n__webpack_require__.p += 'first/';`;
                }
              },
            );
            compilation.hooks.runtimeModule.tap(
              { name: 'SecondSourceUpdate', stage: 10 },
              (module) => {
                if (!module.source) return;
                if (module.name === 'source-updates') {
                  if (!module.generate().includes('__webpack_require__.nc')) {
                    throw new Error(
                      'Hooks must receive the preceding raw source',
                    );
                  }
                  module.source.source = `
                  "use strict";
                  var loader = __webpack_require__;
                  globalThis.runtimeSourceUpdates = {
                    loaderType: typeof loader,
                    read: function() { return __webpack_require__.p; },
                    write: function(value) { __webpack_require__.p = value; },
                    defineType: function() { return typeof __webpack_require__.d; },
                    strict: (function() { return this; })() === undefined
                  };
                `;
                }
                if (module.constructor.name === chunkLoadingConstructor) {
                  module.source.source = `${module.generate()}\n__webpack_require__.p += 'second/';`;
                }
              },
            );
            compilation.hooks.afterProcessAssets.tap(
              'RuntimeSourceUpdates',
              () => {
                if (!modifiedChunkLoading) {
                  throw new Error(
                    'The case must modify the chunk loading runtime',
                  );
                }
              },
            );
          },
        );
      }),
    ],
  }),
);
