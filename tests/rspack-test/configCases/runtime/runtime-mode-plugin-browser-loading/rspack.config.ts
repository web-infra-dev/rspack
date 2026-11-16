import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'web',
  devtool: false,
  module: { rules: [{ test: /\.css$/, type: 'css/auto' }] },
  output: {
    publicPath: '',
    chunkFilename: '[name].js',
    cssChunkFilename: '[name].css',
  },
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
    concatenateModules: false,
  },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeGlobals, RuntimeModule, RuntimePlugin } = compiler.rspack;
      const rstestScope =
        compiler.options.experiments.runtimeMode === 'rspack'
          ? RuntimeGlobals.requireScope
          : RuntimeGlobals.require;
      class BrowserLoading extends RuntimeModule {
        constructor() {
          super('plugin browser loading', RuntimeModule.STAGE_TRIGGER);
        }
        generate() {
          return `
          __webpack_require__.webExtRt = {
            scripts: [], styles: [], failures: [], jsonp: [], retried: {},
            resolveScriptUrl: url => {
              var resolved = new URL(url);
              resolved.searchParams.set('bridge', '1');
              resolved.hash = 'bridge';
              return resolved.href;
            }
          };
          __webpack_require__.sriHashes = { lazy: 'test-integrity' };
          __webpack_require__.miniCssF = function (id) { return __webpack_require__.k(id); };
          __webpack_require__.gca = function (id) { return __webpack_require__.p + __webpack_require__.u(id); };
          __webpack_require__.rbLoadStyleSheet = function (href) {
            __webpack_require__.webExtRt.styles.push(href);
            return href;
          };
          var getFilename = __webpack_require__.u;
          __webpack_require__.u = function (id) {
            return getFilename(id) + (__webpack_require__.webExtRt.retried[id] ? '?retry=1' : '');
          };
          var load = __webpack_require__.e;
          // Nuxt reports the original error and lets the caller decide whether to retry.
          var reportLoad = __webpack_require__.e = function (id) {
            return load(id).catch(function (error) {
              var event = new Event('nuxt:preloadError', { cancelable: true });
              event.payload = error;
              document.defaultView.dispatchEvent(event);
              throw error;
            });
          };
          __webpack_require__.e = function ensureChunk(id) {
            return reportLoad(id).catch(function (error) {
              __webpack_require__.webExtRt.failures.push(error);
              if (__webpack_require__.webExtRt.retried[id]) throw error;
              __webpack_require__.webExtRt.retried[id] = true;
              return reportLoad(id);
            });
          };
          __webpack_require__.f.miniCss = function(id) { __webpack_require__.miniCssF(id); };
          // Web extensions may install an individual handler. The complete
          // table is an object; to preload whole chunks, call ensureChunk.
          __webpack_require__.webExtRt.eagerCalls = [];
          __webpack_require__.f.i = id => { __webpack_require__.webExtRt.eagerCalls.push(id); };
          __webpack_require__.webExtRt.prewarm = ids => Promise.all(ids.map(id => __webpack_require__.e(id)));
          __webpack_require__.F.plugin = id => 'prefetch:' + id;
          __webpack_require__.H.plugin = id => 'preload:' + id;
          __webpack_require__.hmrM = () => [__webpack_require__.hmrF(), __webpack_require__.h()];
        `;
        }
      }
      compiler.hooks.thisCompilation.tap('BrowserLoading', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'BrowserLoading',
          (_chunk, set) => {
            for (const field of [
              RuntimeGlobals.loadScript,
              RuntimeGlobals.getChunkCssFilename,
              RuntimeGlobals.prefetchChunk,
              RuntimeGlobals.prefetchChunkHandlers,
              RuntimeGlobals.preloadChunk,
              RuntimeGlobals.preloadChunkHandlers,
              RuntimeGlobals.getFullHash,
              RuntimeGlobals.getUpdateManifestFilename,
              RuntimeGlobals.hmrDownloadManifest,
              RuntimeGlobals.baseURI,
              RuntimeGlobals.moduleFactories,
              RuntimeGlobals.require,
            ])
              set.add(field);
          },
        );
        compilation.hooks.runtimeRequirementInTree
          .for(RuntimeGlobals.ensureChunkHandlers)
          .tap('BrowserLoading', (chunk) => {
            compilation.addRuntimeModule(chunk, new BrowserLoading());
          });
        // Remotion's script bridge and the SRI plugin use this public waterfall.
        RuntimePlugin.getCompilationHooks(compilation).createScript.tap(
          'BrowserLoading',
          (code) =>
            code +
            '\nscript.integrity = __webpack_require__.sriHashes[chunkId];\nscript.crossOrigin = "anonymous";\nscript.src = __webpack_require__.webExtRt.resolveScriptUrl(script.src);',
        );
        compilation.hooks.runtimeModule.tap('BrowserLoading', (mod) => {
          if (!mod.source) return;
          const original = mod.generate.bind(mod);
          if (mod.name === 'load_script') {
            // Wrap the original loader (Fluent UI / Web Extend); Grafana guards
            // an individual statement in the generated script creation code.
            const wrapLoader = () =>
              original().replace(
                /script\.integrity = [^;]+;\s*script\.crossOrigin = [^;]+;/,
                'if (window.pluginSriEnabled) { $& }',
              ) +
              `
            var nativeLoad = __webpack_require__.l;
            __webpack_require__.l = function (url, done, key, id) {
              __webpack_require__.webExtRt.scripts.push(url);
              return nativeLoad(url, done, key, id);
            };`;
            if (compiler.options.experiments.runtimeMode === 'rspack') {
              mod.generate = wrapLoader;
            } else {
              mod.source.source = Buffer.from(wrapLoader());
            }
          } else if (
            mod.constructor.name === 'JsonpChunkLoadingRuntimeModule'
          ) {
            // Rstest preserves mocks when an asynchronously loaded factory already exists.
            mod.source.source =
              original().replace(
                /for \((?:var )?moduleId in moreModules\) \{/g,
                `$&\nif (${rstestScope}.rstest_original_module_factories[moduleId]) continue;`,
              ) +
              `
            var loadJsonp = __webpack_require__.f.j;
            __webpack_require__.f.j = function (id, promises) {
              __webpack_require__.webExtRt.jsonp.push(id);
              return loadJsonp(id, promises);
            };`;
          } else if (mod.name === 'css_loading') {
            // Route the actual stylesheet insertion through a plugin-owned target.
            mod.source.source = Buffer.from(
              original().replace(
                'document.head.appendChild(link)',
                '(__webpack_require__.rbLoadStyleSheet(link.href), document.head.appendChild(link))',
              ),
            );
          } else if (
            mod.name === 'get javascript chunk filename' ||
            mod.name === 'get css chunk filename'
          ) {
            const [field, ext] = mod.name.startsWith('get javascript')
              ? [RuntimeGlobals.getChunkScriptFilename, 'js']
              : [RuntimeGlobals.getChunkCssFilename, 'css'];
            mod.source.source = `${field} = id => (id === "probe" ? "plugin-output" : id) + ".${ext}";`;
          }
        });
      });
    }),
  ],
});
