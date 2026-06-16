const { RuntimeModule } = require('@rspack/core');

class CustomRuntimeModule extends RuntimeModule {
  constructor() {
    super('custom');
  }

  generate() {
    return '__webpack_require__.custom = 1;';
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    runtimeMode: 'rspack',
  },
  plugins: [
    (compiler) => {
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
    },
  ],
};
