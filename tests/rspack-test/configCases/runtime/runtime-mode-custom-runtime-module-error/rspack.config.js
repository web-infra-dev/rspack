const { RuntimeModule } = require('@rspack/core');

class CustomRuntimeModule extends RuntimeModule {
  constructor() {
    super('custom');
  }

  generate() {
    return `
const originalRequire = __webpack_require__;
__webpack_require__ = function(...args) {
  return originalRequire(...args);
};
for (const key in originalRequire) {
  __webpack_require__[key] = originalRequire[key];
}
function localShadow(__webpack_require__) {
  return __webpack_require__.custom;
}
__webpack_require__.custom = 1;
globalThis.__custom_runtime_module_value__ = __webpack_require__.custom;
globalThis.__custom_runtime_module_shadow__ = localShadow({ custom: 2 });
`;
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
