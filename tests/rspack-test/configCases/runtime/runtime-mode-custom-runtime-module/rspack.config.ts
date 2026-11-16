import { defineConfig, definePlugin } from '@rspack/cli';
import { RuntimeModule } from '@rspack/core';

class CustomRuntimeModule extends RuntimeModule {
  constructor() {
    super('custom');
  }

  generate() {
    // Lynx emits this optional callback even in builds without onChunksLoaded.
    return `var state = __webpack_require__.custom = {
        intercepted: false, onChunksLoaded: typeof __webpack_require__.O
      };
      __webpack_require__.i.push(function () { state.intercepted = true; });
      if (false) __webpack_require__.O();`;
  }
}

export default defineConfig({
  experiments: {
    runtimeMode: 'rspack',
  },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeGlobals } = compiler.rspack;

      compiler.hooks.thisCompilation.tap(
        'CustomRuntimeModulePlugin',
        (compilation) => {
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            'CustomRuntimeModulePlugin',
            (chunk, runtimeRequirements) => {
              runtimeRequirements.add(RuntimeGlobals.require);
              compilation.addRuntimeModule(chunk, new CustomRuntimeModule());
            },
          );
        },
      );
    }),
  ],
});
