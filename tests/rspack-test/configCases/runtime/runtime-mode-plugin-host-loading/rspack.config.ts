import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  devtool: false,
  output: {
    publicPath: 'https://example.test/',
    chunkLoading: 'require',
    chunkFilename: '[name].cjs',
  },
  optimization: { moduleIds: 'named', chunkIds: 'named' },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeModule, RuntimeGlobals: globals } = compiler.rspack;
      class HostLoadingRuntime extends RuntimeModule {
        constructor() {
          super('host chunk loading', RuntimeModule.STAGE_TRIGGER);
        }
        generate() {
          const chunks = Object.fromEntries(
            [...this.chunk!.getAllAsyncChunks()].map((chunk) => {
              const [module] = this.chunkGraph!.getChunkModules(chunk);
              return [
                chunk.name,
                {
                  id: chunk.id,
                  module: this.chunkGraph!.getModuleId(module),
                  url: this.compilation!.getPath('[name].cjs', { chunk }),
                },
              ];
            }),
          );
          return `
          var locales = ${JSON.stringify(chunks)};
          __webpack_require__.lynx_aci = Object.fromEntries(
            Object.values(locales).map(chunk => [chunk.id, chunk.url])
          );
          // Lynx and AngularLynx install a host's evaluated chunk into the current runtime.
          globalThis.processEvalResultByHost = {
            main: (getEvalResult, schema) => {
              var result = getEvalResult(schema);
              __webpack_require__.C(result);
              return result;
            }
          };
          __webpack_require__.f.require = function (id, promises) {
            var url = __webpack_require__.p + (__webpack_require__.lynx_aci[id] || __webpack_require__.u(id));
            promises.push(lynx.requireModuleAsync(url).then(result =>
              processEvalResultByHost.main(() => result, url)
            ));
          };
          // The localization plugin uses the normal ensure function for its fallback locale.
          __webpack_require__.loadTranslations = (locale) => {
            var chunk = locales[locale] || locales.en;
            return __webpack_require__.e(chunk.id).then(() => __webpack_require__(chunk.module));
          };
          // Lynx externals load the main-thread and background sections by URL.
          __webpack_require__.readExternal = section => {
            var response = lynx.fetchBundle(__webpack_require__.p + 'external.bundle').wait();
            return lynx.loadScript(section, {bundleName: response.url});
          };
          // Cache startup events and clear them after the wrapped startup completes.
          var events = __webpack_require__.lynx_ce = {loaded: false, cachedActions: []};
          var startup = __webpack_require__.x;
          __webpack_require__.x = () => Promise.resolve(startup()).finally(() => {
            events.loaded = true;
            events.cachedActions.length = 0;
          });`;
        }
      }
      compiler.hooks.thisCompilation.tap('HostLoading', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'HostLoading',
          (_chunk, requirements) => {
            requirements.add(globals.externalInstallChunk);
            requirements.add(globals.publicPath);
            requirements.add(globals.startup);
          },
        );
        compilation.hooks.runtimeRequirementInTree
          .for(globals.ensureChunkHandlers)
          .tap('HostLoading', (chunk) => {
            compilation.addRuntimeModule(chunk, new HostLoadingRuntime());
          });
      });
    }),
  ],
});
