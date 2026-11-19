import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  target: 'node',
  entry: { active: './index.js', plain: './plain.js' },
  output: { filename: '[name].js', publicPath: '/initial/' },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeGlobals, RuntimeModule } = compiler.rspack;
      compiler.hooks.thisCompilation.tap('guard', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'guard',
          (chunk, requirements) => {
            if (chunk.name !== 'active') return;
            requirements.add(RuntimeGlobals.publicPath);
            requirements.add(RuntimeGlobals.getChunkScriptFilename);
            compilation.addRuntimeModule(
              chunk,
              new (class extends RuntimeModule {
                constructor() {
                  super('00 wrap the loader');
                }
                generate() {
                  return `var original = __webpack_require__;
var before = Object.keys(original);
function guarded(id) { return original(id); }
before.forEach(function(key) { guarded[key] = original[key]; });
__webpack_require__ = guarded;
guarded.privateState = { before: before };`;
                }
              })(),
            );
          },
        );
        compilation.hooks.processAssets.tap('guard', (assets) => {
          const hasBridge = (name: string) =>
            assets[name]
              .source()
              .toString()
              .includes('// webpack runtime module bridge');
          expect(hasBridge('plain.js')).toBe(false);
          expect(hasBridge('active.js')).toBe(
            compiler.options.experiments.runtimeMode === 'rspack',
          );
        });
      });
    }),
  ],
});
