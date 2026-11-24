import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

export default [false, true].map((esm, index) =>
  defineConfig({
    mode: 'development',
    target: 'node',
    devtool: false,
    entry: './index.js',
    module: {
      rules: [
        {
          test: /asset\.txt$/,
          use: path.join(import.meta.dirname, 'asset-loader.cjs'),
        },
      ],
    },
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
            // Rstest proxies the original loader so helper and cache identity is kept.
            var calls = [];
            __webpack_require__ = new Proxy(__webpack_require__, {
              apply(target, receiver, args) {
                calls.push(args[0]);
                return Reflect.apply(target, receiver, args);
              }
            });
            __webpack_require__.rstestProxyCalls = calls;
            __webpack_require__.$Refresh$ = { seen: [] };
            var intercept = function (options) {
              // Lynx refresh can select the main-thread factory for a background module ID.
              if (options.id === './setup.cjs(react:background)') {
                options.id = options.id.replace('(react:background)', '(react:main-thread)');
                options.factory = __webpack_modules__[options.id];
              }
              var original = options.factory;
              options.factory = function (module, exports, require) {
                __webpack_require__.$Refresh$.seen.push(module.id);
                var previous = globalThis.$RefreshReg$;
                globalThis.$RefreshReg$ = () => module.id;
                try {
                  return original.call(this, module, exports, require);
                } finally {
                  globalThis.$RefreshReg$ = previous;
                }
              };
            };
            __webpack_require__.$Refresh$.intercept = intercept;
            __webpack_require__.$Refresh$.loadFromBackground = function (id) {
              __webpack_modules__[id + '(react:main-thread)'] = __webpack_modules__[id];
              return __webpack_require__(id + '(react:background)');
            };
            __webpack_require__.i.push(intercept);
            var registrations = __webpack_require__.rstestIds = { setups: [], tests: {}, browser: {} };
            __webpack_require__.rstest_register_setup_id = id => registrations.setups.push(id);
            __webpack_require__.rstest_register_test_entry_id = (name, id) => registrations.tests[name] = id;
            __webpack_require__.rstest_register_browser_test_entry = (name, id) => registrations.browser[name] = id;
            __webpack_require__.rstestCacheEntry = function (id) {
              return [__webpack_module_cache__[id], __webpack_require__.c[id], __webpack_modules__[id], __webpack_require__.m[id]];
            };
            __webpack_require__.rstestResetModule = function (id) {
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
            // Aurelia exposes its current webpack loader to application code.
            globalThis.aureliaWebpackRequire = __webpack_require__;
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
            (_chunk, set) => {
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
            },
          );
          // Expensify registers its factory timing module using the compiler's hook key.
          compilation.hooks.runtimeRequirementInTree
            .for(g.interceptModuleExecution)
            .tap('ModuleExecution', (chunk) => {
              compilation.addRuntimeModule(chunk, new ModuleExecutionRuntime());
            });
        });
      }),
    ],
  }),
);
