import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  experiments: { runtimeMode: 'rspack' },
  plugins: [
    definePlugin((compiler) => {
      let generation = 0;
      class CustomRuntime extends compiler.rspack.RuntimeModule {
        constructor() {
          super(
            'custom-hot-update',
            compiler.rspack.RuntimeModule.STAGE_TRIGGER,
          );
        }
        generate() {
          return `__webpack_require__.customUpdate = "generation-${generation}";
          __webpack_require__.getRuntimeRequire = () => __webpack_require__;
          __webpack_require__.getRuntimePath = () => __webpack_require__.p;
          __webpack_require__.setRuntimePath = value => { __webpack_require__.p = value; };`;
        }
      }
      compiler.hooks.thisCompilation.tap('CustomRuntime', (compilation) => {
        generation++;
        compilation.hooks.runtimeModule.tap('CustomRuntime', (module) => {
          if (
            module.constructor.name === 'NonceRuntimeModule' &&
            module.source
          ) {
            module.source.source = `${module.generate()}\n__webpack_require__.nc = "hook-${generation}";`;
          }
        });
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'CustomRuntime',
          (chunk) => {
            compilation.addRuntimeModule(chunk, new CustomRuntime());
          },
        );
      });
    }),
  ],
});
