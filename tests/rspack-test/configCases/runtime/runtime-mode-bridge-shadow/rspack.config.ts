import { defineConfig, definePlugin } from '@rspack/cli';
export default defineConfig({
  target: 'node',
  experiments: { runtimeMode: 'rspack' },
  optimization: { runtimeChunk: 'single', concatenateModules: false },
  output: {
    module: true,
    library: { type: 'modern-module' },
    filename: '[name].mjs',
  },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeModule } = compiler.rspack;
      compiler.hooks.thisCompilation.tap('shadow', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'shadow',
          (chunk) => {
            compilation.addRuntimeModule(
              chunk,
              new (class extends RuntimeModule {
                constructor() {
                  super('shadowed require');
                }
                generate() {
                  return `// __rspack_context is intentionally only in a comment.
            function read(__webpack_require__) { return __webpack_require__.p; }
            globalThis.runtimeShadowProbe = [read({p: 7}), '__webpack_require__'];`;
                }
              })(),
            );
            for (const child of chunk.getAllAsyncChunks()) {
              compilation.addRuntimeModule(
                child,
                new (class extends RuntimeModule {
                  constructor() {
                    super('no runtime globals');
                  }
                  generate() {
                    return 'globalThis.shadowAsyncRan = true;';
                  }
                })(),
              );
            }
          },
        );
        compilation.hooks.processAssets.tap('shadow', (assets) => {
          for (const asset of Object.values(assets)) {
            if (
              asset
                .source()
                .toString()
                .includes('// webpack runtime module bridge')
            ) {
              throw new Error(
                'a local binding must not enable the runtime bridge',
              );
            }
          }
        });
      });
    }),
  ],
});
