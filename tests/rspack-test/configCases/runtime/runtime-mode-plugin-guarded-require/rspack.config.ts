import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeModule } = compiler.rspack;
      compiler.hooks.thisCompilation.tap('GuardedRequire', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'GuardedRequire',
          (chunk) => {
            const runtime = new RuntimeModule(
              'Re.Pack guarded require',
              RuntimeModule.STAGE_NORMAL,
            );
            runtime.generate = () => `
              var original = __webpack_require__;
              function guardedRequire(id) {
                try {
                  return original(id);
                } catch (error) {
                  if (error.name === 'ChunkLoadError' || !globalThis.ErrorUtils) throw error;
                  globalThis.ErrorUtils.reportFatalError(error);
                }
              }
              for (var key of Object.getOwnPropertyNames(original)) {
                guardedRequire[key] = original[key];
              }
              __webpack_require__ = guardedRequire;`;
            compilation.addRuntimeModule(chunk, runtime);
          },
        );
      });
    }),
  ],
});
