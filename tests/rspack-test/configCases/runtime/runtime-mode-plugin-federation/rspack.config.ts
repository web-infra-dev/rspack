import { defineConfig, definePlugin } from '@rspack/cli';

export default [false, true].map((ownStartup, index) =>
  defineConfig({
    mode: 'development',
    target: 'node',
    entry: './index.js',
    devtool: false,
    optimization: {
      moduleIds: 'named',
      chunkIds: 'named',
      concatenateModules: false,
    },
    output: {
      filename: `bundle${index}.js`,
      chunkFilename: `[name]${index}.js`,
    },
    plugins: [
      definePlugin((compiler) => {
        const { RuntimeModule, RuntimeGlobals: g } = compiler.rspack;
        class FederationRuntime extends RuntimeModule {
          constructor() {
            super('plugin federation handlers', RuntimeModule.STAGE_TRIGGER);
          }
          generate() {
            const entry = [
              ...this.chunkGraph!.getChunkEntryModulesIterable(this.chunk!),
            ][0];
            const entryId = this.chunkGraph!.getModuleId(entry);
            return `
          __webpack_require__.federation = { calls: [] };
          __webpack_require__.remotesLoadingData = { kind: 'remote' };
          __webpack_require__.consumesLoadingData = { kind: 'shared' };
          __webpack_require__.S = { default: {} };
          __webpack_require__.I = function (name) {
            return __webpack_require__.S[name];
          };
          __webpack_require__.f.remotes = function (id, promises) {
            __webpack_require__.federation.calls.push([__webpack_require__.remotesLoadingData.kind, id]);
            promises.push(Promise.resolve());
          };
          __webpack_require__.f.consumes = function (id, promises) {
            __webpack_require__.federation.calls.push([__webpack_require__.consumesLoadingData.kind, id]);
            __webpack_require__.I('default')[id] = true;
          };
          // Rstest keeps the original in-memory chunk handler when MF tries to replace it.
          var handlers = __webpack_require__.f;
          var keep = handlers.require || handlers.readFileVm;
          var key = handlers.require ? 'require' : 'readFileVm';
          __webpack_require__.f = new Proxy(handlers, {
            set(target, name, value) {
              if (name === key && target[name]) return true;
              return Reflect.set(target, name, value);
            }
          });
          __webpack_require__.f[key] = function () { throw new Error('unexpected native loader'); };
          __webpack_require__.federation.keptLoader = typeof keep === 'function' && __webpack_require__.f[key] === keep;
          var previousStartup = __webpack_require__.x;
          __webpack_require__.x = function () {
            __webpack_require__.federation.started = true;
            ${ownStartup ? `return __webpack_require__(__webpack_require__.s = ${JSON.stringify(entryId)});` : 'return previousStartup();'}
          };
          __webpack_require__.federation.runtime = function () {
            __webpack_require__.O(undefined, ['remote'], function () { return 'remote ready'; });
            var deferredResult = __webpack_require__.O();
            var entryResult = __webpack_require__.X(undefined, ['remote'], function () { return 'entry ready'; });
            return [__webpack_require__.j, __webpack_require__.s, entryResult, deferredResult];
          };
        `;
          }
        }
        compiler.hooks.thisCompilation.tap(
          'FederationRuntime',
          (compilation) => {
            compilation.hooks.additionalTreeRuntimeRequirements.tap(
              'FederationRuntime',
              (chunk, set) => {
                for (const field of [
                  g.ensureChunk,
                  g.ensureChunkHandlers,
                  g.startup,
                  g.startupEntrypoint,
                  g.onChunksLoaded,
                  g.entryModuleId,
                  g.runtimeId,
                ])
                  set.add(field);
                if (ownStartup) set.add(g.startupNoDefault);
                compilation.addRuntimeModule(chunk, new FederationRuntime());
              },
            );
          },
        );
      }),
    ],
  }),
);
