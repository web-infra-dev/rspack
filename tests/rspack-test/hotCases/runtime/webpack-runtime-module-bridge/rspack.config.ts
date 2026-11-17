import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeModule, RuntimeGlobals } = compiler.rspack;
      let revision = 0;
      compiler.hooks.thisCompilation.tap('runtime update', (compilation) => {
        const build = ++revision;
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'runtime update',
          (chunk, requirements) => {
            requirements.add(RuntimeGlobals.getFullHash);
            compilation.addRuntimeModule(
              chunk,
              new (class extends RuntimeModule {
                constructor() {
                  super('plugin revision', RuntimeModule.STAGE_TRIGGER);
                  this.fullHash = true;
                }
                generate() {
                  // Keep the loader captured before HMR so post-update writes exercise the original bridge.
                  const captureLoader =
                    build === 1 ? ' loader.initialPluginLoader = loader;' : '';
                  return `var loader = __webpack_require__; loader.pluginRevision = ${build}; loader.pluginHash = ${JSON.stringify(compilation.hash)};${captureLoader}`;
                }
              })(),
            );
          },
        );
      });
    }),
  ],
});
