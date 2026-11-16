import { defineConfig, definePlugin } from '@rspack/cli';

export default [false, true].map((esm, index) =>
  defineConfig({
    mode: 'development',
    target: 'node',
    devtool: false,
    entry: './index.js',
    optimization: {
      concatenateModules: false,
      runtimeChunk: esm ? 'single' : false,
    },
    output: {
      filename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      ...(esm ? { module: true, library: { type: 'modern-module' } } : {}),
    },
    plugins: [
      definePlugin((compiler) => {
        const { RuntimeGlobals: g, RuntimeModule } = compiler.rspack;
        class ModuleExecutionRuntime extends RuntimeModule {
          constructor() {
            super('react refresh', RuntimeModule.STAGE_TRIGGER);
          }
          generate() {
            return `
            __webpack_require__.$Refresh$ = { seen: [] };
            var intercept = function (options) {
              var original = options.factory;
              options.factory = function (module, exports, require) {
                __webpack_require__.$Refresh$.seen.push(module.id);
                return original.call(this, module, exports, require);
              };
            };
            __webpack_require__.$Refresh$.intercept = intercept;
            __webpack_require__.i.push(intercept);
            __webpack_require__.rstest_register_setup_id = function (id) {
              return [__webpack_module_cache__[id], __webpack_require__.c[id], __webpack_modules__[id], __webpack_require__.m[id]];
            };
            __webpack_require__.rstest_register_browser_test_entry = function (id) {
              delete __webpack_module_cache__[id];
              return __webpack_require__(id);
            };
            // WxsTemplatePlugin replaces the namespace and default interop helpers.
            var originalNamespace = __webpack_require__.r;
            var originalDefault = __webpack_require__.n;
            __webpack_require__.r = function (value) {
              originalNamespace(value);
              value.pluginNamespace = true;
            };
            __webpack_require__.n = function (value) {
              var getter = originalDefault(value);
              getter.fromPlugin = true;
              return getter;
            };
            __webpack_require__.ab = 'native-assets/';
            __webpack_require__.pluginExports = function (value) {
              var ns = {};
              __webpack_require__.r(ns);
              __webpack_require__.d(ns, { value: function () { return value; } });
              var getter = __webpack_require__.n(ns);
              return [ns.pluginNamespace, ns.value, getter.fromPlugin, __webpack_require__.g === globalThis];
            };
          `;
          }
        }
        compiler.hooks.thisCompilation.tap('ModuleExecution', (compilation) => {
          compilation.hooks.runtimeModule.tap('SafeRefresh', (mod) => {
            if (mod.name === 'react refresh' && mod.source) {
              mod.source.source = Buffer.from(
                mod.generate().replace(
                  'var original = options.factory;',
                  `var original = options.factory;
               if (typeof original !== 'function') {
                 options.factory = function (_module, exports) { return exports; };
                 return;
               }`,
                ),
              );
            }
          });
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            'ModuleExecution',
            (chunk, set) => {
              for (const field of [
                g.moduleCache,
                g.moduleFactories,
                g.interceptModuleExecution,
                g.makeNamespaceObject,
                g.compatGetDefaultExport,
                g.definePropertyGetters,
                g.global,
                g.moduleId,
              ])
                set.add(field);
              compilation.addRuntimeModule(chunk, new ModuleExecutionRuntime());
            },
          );
        });
      }),
    ],
  }),
);
