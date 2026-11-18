import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeModule, RuntimeGlobals } = compiler.rspack;
      let revision = 0;
      let previousHash: string | null = null;
      compiler.hooks.done.tap('runtime update', (stats) => {
        previousHash = stats.hash;
      });
      compiler.hooks.thisCompilation.tap('runtime update', (compilation) => {
        const build = ++revision;
        const stylesheetHash = previousHash ? `.${previousHash}` : '';
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
                  // Lynx's CSS host reads paths that are keyed by the previous build's hash.
                  const chunkName = this.chunk!.name;
                  const stylesheet = `${chunkName}${stylesheetHash}.css.hot-update.json`;
                  const source = `var loader = __webpack_require__; loader.pluginRevision = ${build}; loader.pluginHash = ${JSON.stringify(compilation.hash)};${captureLoader}`;
                  return `${source}\nloader.cssHotUpdateList = ${JSON.stringify([[chunkName, stylesheet]])};`;
                }
              })(),
            );
          },
        );
      });
    }),
  ],
});
