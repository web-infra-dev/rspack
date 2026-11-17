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
                  return `var loader = __webpack_require__; loader.pluginRevision = ${build}; loader.pluginHash = ${JSON.stringify(compilation.hash)};`;
                }
              })(),
            );
          },
        );
      });
    }),
  ],
});
