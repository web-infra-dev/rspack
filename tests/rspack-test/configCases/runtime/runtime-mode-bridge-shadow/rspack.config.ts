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
        let generateCalls = 0;
        const observedSources: string[] = [];
        // Analysis plugins such as Rsdoctor only read the generated source.
        compilation.hooks.runtimeModule.tap('observe runtime', (mod) => {
          observedSources.push(mod.generate());
        });
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
                  generateCalls++;
                  return `// __rspack_context is intentionally only in a comment.
            function read(__webpack_require__) { return __webpack_require__.p; }
            globalThis.runtimeShadowProbe = [read({p: 7}), '__webpack_require__'];
            // Lynx's runtime config module has only a host-side effect.
            globalThis.lynx ||= {};
            lynx.__runtime_configs__ = { audit: 42 };`;
                }
              })(),
            );
          },
        );
        compilation.hooks.processAssets.tap('shadow', (assets) => {
          expect(generateCalls).toBe(1);
          expect(
            observedSources.some((source) =>
              source.includes('lynx.__runtime_configs__'),
            ),
          ).toBe(true);
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
