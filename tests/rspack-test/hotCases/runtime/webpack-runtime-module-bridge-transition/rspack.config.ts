import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  output: { publicPath: 'https://test.cases/path/' },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeModule, RuntimeGlobals } = compiler.rspack;
      let revision = 0;
      compiler.hooks.thisCompilation.tap('runtime upgrade', (compilation) => {
        const build = ++revision;
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'runtime upgrade',
          (chunk, requirements) => {
            requirements.add(RuntimeGlobals.publicPath);
            compilation.addRuntimeModule(
              chunk,
              new (class extends RuntimeModule {
                constructor() {
                  super('runtime upgrade', RuntimeModule.STAGE_TRIGGER);
                }
                generate() {
                  const revision = `globalThis.bridgeRevision = ${build};`;
                  if (build !== 2) return revision;
                  return `${revision}
                  __webpack_require__.p = "https://example.com/";
                  __webpack_require__.i.push(function(options) {
                    if (options.id !== "./subject.cjs") return;
                    var factory = options.factory;
                    options.factory = function(module, exports, incoming) {
                      function require(id) { return incoming(id); }
                      require.fromInterceptor = 41;
                      factory.call(this, module, exports, require);
                      require.afterFactory = 42;
                    };
                  });`;
                }
              })(),
            );
          },
        );
      });
    }),
  ],
});
