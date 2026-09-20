import { defineConfig, definePlugin } from '@rspack/cli';

import { RuntimeModule } from '@rspack/core';

class CustomRuntimeModule extends RuntimeModule {
  constructor() {
    super('custom');
  }

  generate() {
    return '__webpack_require__.custom = 1;';
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
