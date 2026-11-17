import { defineConfig, definePlugin } from '@rspack/cli';

export default [false, true].map((esm, index) =>
  defineConfig({
    target: 'node',
    mode: 'development',
    devtool: false,
    optimization: {
      concatenateModules: false,
      moduleIds: 'named',
      runtimeChunk: esm ? 'single' : false,
    },
    output: {
      publicPath: '/initial/',
      chunkFilename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      filename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      ...(esm ? { module: true, library: { type: 'modern-module' } } : {}),
    },
    plugins: [
      definePlugin((compiler) => {
        const { RuntimeGlobals: g, RuntimeModule } = compiler.rspack;
        let adding = false;
        const hookSource =
          "/* plugin source */ __webpack_require__['fromHook'] = 42;";
        new compiler.rspack.DefinePlugin({
          EXPECT_GENERATE_OVERRIDE:
            compiler.options.experiments.runtimeMode === 'rspack',
          EXPECT_ASYNC_RUNTIME:
            !esm || compiler.options.experiments.runtimeMode === 'rspack',
        }).apply(compiler);
        class Extension extends RuntimeModule {
          constructor() {
            super('webpack extension', RuntimeModule.STAGE_TRIGGER);
          }
          generate() {
            if (adding) throw new Error('generate ran during addRuntimeModule');
            return `
            const external = __webpack_require__;
            const key = ['private', 'field'].join('_');
            Object.defineProperty(external, key, { value: 17, configurable: true });
            external.probe = function () {
              const { p: previous } = external;
              ({ path: external['p'] } = { path: '/changed/' });
              return [previous, external[key], Object.getOwnPropertyDescriptor(external, key).value,
                __webpack_modules__ === external.m, __webpack_module_cache__ === external.c, external.O === undefined];
            };
            ${g.interceptModuleExecution}.push(function (o) {
              if (!o.id.endsWith('/subject.cjs')) return;
              if (o.factory !== external.m[o.id]) throw new Error('factory identity');
              const original = o.factory;
              o.factory = function (module, exports, incoming) {
                function hotRequire(id) { return incoming(id); }
                Object.defineProperty(hotRequire, '$Refresh$', { value: module.id });
                var result = original.call(this, module, exports, hotRequire);
                if (hotRequire.writtenByFactory !== 7) throw new Error('private factory write');
                hotRequire.afterFactory = 42;
                return result;
              };
            });
          `;
          }
        }
        compiler.hooks.thisCompilation.tap('bridge', (compilation) => {
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            'bridge',
            (chunk, requirements) => {
              for (const global of [
                g.publicPath,
                g.moduleId,
                g.require,
                g.moduleFactories,
                g.moduleCache,
                g.interceptModuleExecution,
              ])
                requirements.add(global);
              adding = true;
              compilation.addRuntimeModule(chunk, new Extension());
              adding = false;
              for (const child of chunk.getAllAsyncChunks()) {
                if (child.name !== 'child') continue;
                compilation.addRuntimeModule(
                  child,
                  new (class extends RuntimeModule {
                    constructor() {
                      super('async extension');
                    }
                    generate() {
                      return `var alias = __webpack_require__; ({ value: alias.p } = { value: '/async/' });`;
                    }
                  })(),
                );
              }
            },
          );
          compilation.hooks.runtimeRequirementInTree
            .for(g.publicPath)
            .tap('public path hook', (chunk) => {
              compilation.addRuntimeModule(
                chunk,
                new (class extends RuntimeModule {
                  constructor() {
                    super('public path hook');
                  }
                  generate() {
                    return '__webpack_require__.fromRequirements = 1;';
                  }
                })(),
              );
            });
          compilation.hooks.processAssets.tap('bridge', (assets) => {
            expect(
              Object.values(assets).some((asset) =>
                asset.source().toString().includes(hookSource),
              ),
            ).toBe(true);
          });
          compilation.hooks.runtimeModule.tap('bridge', (mod) => {
            if (
              mod.name === 'public path hook' &&
              compiler.options.experiments.runtimeMode === 'rspack'
            ) {
              const original = mod.generate.bind(mod);
              mod.generate = () =>
                original() + '__webpack_require__.fromGenerate = 2;';
            }
            if (
              mod.constructor.name !== 'PublicPathRuntimeModule' ||
              !mod.source
            )
              return;
            // Keep the native declarations at their original stage and in their original scope.
            const generated = mod.generate();
            mod.source.source = generated + '\n' + hookSource;
          });
        });
      }),
    ],
  }),
);
