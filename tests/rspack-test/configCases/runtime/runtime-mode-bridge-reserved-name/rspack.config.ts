import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'node',
  optimization: { runtimeChunk: false, concatenateModules: true },
  output: {
    module: true,
    library: { type: 'modern-module' },
    filename: 'bundle.mjs',
    publicPath: '/original/',
  },
  plugins: [
    definePlugin((compiler) => {
      const { RuntimeGlobals, RuntimeModule } = compiler.rspack;
      compiler.hooks.thisCompilation.tap('public path', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'public path',
          (chunk, requirements) => {
            requirements.add(RuntimeGlobals.publicPath);
            compilation.addRuntimeModule(
              chunk,
              new (class extends RuntimeModule {
                constructor() {
                  super('public path override', RuntimeModule.STAGE_TRIGGER);
                }
                generate() {
                  return '__webpack_require__.p = "/plugin/";';
                }
              })(),
            );
          },
        );
      });
    }),
  ],
});
