import { defineConfig, definePlugin } from '@rspack/cli';
import { RuntimeModule } from '@rspack/core';

export default [false, true].map((esm, index) =>
  defineConfig({
    mode: 'development',
    target: 'node',
    entry: { main: './index.js' },
    devtool: false,
    experiments: { runtimeMode: 'rspack' },
    output: {
      filename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      chunkFilename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      publicPath: '/initial/',
      ...(esm ? { module: true, library: { type: 'modern-module' } } : {}),
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      runtimeChunk: esm ? 'single' : false,
    },
    plugins: [
      definePlugin((compiler) => {
        const { RuntimeGlobals } = compiler.rspack;
        new compiler.rspack.DefinePlugin({
          ESM_OUTPUT: JSON.stringify(esm),
        }).apply(compiler);
        class SharedRuntime extends RuntimeModule {
          constructor() {
            super('extension-shared');
          }
          shouldIsolate() {
            return false;
          }
          generate() {
            return 'var extensionSharedValue = 42;';
          }
        }
        class ExtensionRuntime extends RuntimeModule {
          constructor() {
            super('extension', RuntimeModule.STAGE_TRIGGER);
          }
          generate() {
            return `
        "use strict";
        var namespace = {};
        __webpack_require__.r(namespace);
        var secondNamespace = {};
        __rspack_context.N(secondNamespace);
        var shadowed = (function(__webpack_require__, __rspack_context) {
          return __webpack_require__.p + __rspack_context.p;
        })({ p: 'local' }, { p: '-context' });
        var originalEnsure = __webpack_require__.e;
        __webpack_require__.e = function(chunkId) {
          __webpack_require__.extension.calls++;
          return originalEnsure(chunkId);
        };
        ${RuntimeGlobals.require}.extension = {
          shared: extensionSharedValue,
          namespace: namespace.__esModule && secondNamespace.__esModule,
          shadowed: shadowed,
          literal: '中文😀__webpack_require__.p',
          strict: (function() { return this; })() === undefined,
          loadScriptType: typeof __webpack_require__.l,
          scopedPath: (function(context, __rspack_context) {
            return context + __rspack_context + __webpack_require__.p;
          })('local-', 'context-'),
          calls: 0,
          ensure: __webpack_require__.e,
          replaceEnsure: function(value) { __webpack_require__.e = value; },
          updateNonce: function(value) {
            __webpack_require__.nc = value;
            var previous = __webpack_require__.nc++;
            var current = ++__rspack_context.nc;
            __webpack_require__.nc *= 2;
            __webpack_require__.nc ||= 99;
            return [previous, current, __webpack_require__.nc];
          },
          getPath: function() { return __webpack_require__['p']; },
          setPath: function(value) { __rspack_context.p = value; }
        };
      `;
          }
        }
        class RemovedRuntime extends RuntimeModule {
          constructor() {
            super('extension-removed');
          }
          generate() {
            return 'throw new Error("removed runtime must not execute");';
          }
        }
        class AsyncRuntime extends RuntimeModule {
          constructor() {
            super('extension-async');
          }
          generate() {
            return `__webpack_require__.p += "async/";
            __webpack_require__.getAsyncPath = () => __webpack_require__.p;
            __webpack_require__.setAsyncPath = value => { __webpack_require__.p = value; };`;
          }
        }
        class GenerateOverrideRuntime extends RuntimeModule {
          constructor() {
            super('extension-generate-override');
          }
          generate() {
            return 'throw new Error("generate override must run");';
          }
        }
        class BufferRuntime extends RuntimeModule {
          constructor() {
            super('extension-buffer');
          }
          generate() {
            return '__webpack_require__.bufferValue = 1;';
          }
        }
        class HashRuntime extends RuntimeModule {
          fullHash = true;
          constructor() {
            super('extension-full-hash');
          }
          generate() {
            return `__webpack_require__.extensionHash = ${JSON.stringify(this.compilation?.hash)};`;
          }
        }
        compiler.hooks.thisCompilation.tap(
          'RuntimeExtensions',
          (compilation) => {
            compilation.hooks.additionalTreeRuntimeRequirements.tap(
              'RuntimeExtensions',
              (chunk) => {
                for (const asyncChunk of chunk.getAllAsyncChunks()) {
                  if (asyncChunk.name === 'extension-async') {
                    compilation.addRuntimeModule(
                      asyncChunk,
                      new AsyncRuntime(),
                    );
                  }
                }
                compilation.addRuntimeModule(chunk, new SharedRuntime());
                compilation.addRuntimeModule(chunk, new ExtensionRuntime());
                compilation.addRuntimeModule(chunk, new RemovedRuntime());
                compilation.addRuntimeModule(
                  chunk,
                  new GenerateOverrideRuntime(),
                );
                compilation.addRuntimeModule(chunk, new BufferRuntime());
                compilation.addRuntimeModule(chunk, new HashRuntime());
              },
            );
            compilation.hooks.runtimeModule.tap(
              'FirstRuntimeSource',
              (module) => {
                if (
                  module.constructor.name === 'NonceRuntimeModule' &&
                  module.source
                ) {
                  module.source.source = `${module.generate()}\n__webpack_require__.setNonce = value => { __webpack_require__.nc = value; };`;
                }
                if (module.name === 'extension-generate-override') {
                  module.generate = () =>
                    '__webpack_require__.generateValue = 2;';
                }
                if (module.name === 'extension-buffer' && module.source) {
                  module.source.source = Buffer.from(module.generate());
                }
                if (module.name === 'extension-removed' && module.source) {
                  module.source.source = '';
                }
                if (
                  module.constructor.name === 'PublicPathRuntimeModule' &&
                  module.source
                ) {
                  module.source.source = `${module.source.source}\n__webpack_require__.p += 'first/';`;
                }
              },
            );
            compilation.hooks.runtimeModule.tap(
              { name: 'SecondRuntimeSource', stage: 10 },
              (module) => {
                if (
                  module.name === 'extension-buffer' &&
                  module.source &&
                  Buffer.isBuffer(module.source.source)
                ) {
                  const source = module.source.source;
                  source[source.indexOf('1')] = '3'.charCodeAt(0);
                }
                if (
                  module.constructor.name === 'PublicPathRuntimeModule' &&
                  module.source
                ) {
                  module.source.source = `${module.source.source}\n__rspack_context.p += 'second/';`;
                }
              },
            );
          },
        );
      }),
    ],
  }),
);
