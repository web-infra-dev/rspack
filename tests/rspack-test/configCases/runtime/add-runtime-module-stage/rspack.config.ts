import { defineConfig, definePlugin } from '@rspack/cli';
import { RuntimeModule } from '@rspack/core';

export default defineConfig({
  entry: './index.js',
  mode: 'development',
  devtool: false,
  optimization: {
    minimize: false,
    sideEffects: false,
    concatenateModules: false,
    usedExports: false,
    innerGraph: false,
    providedExports: false,
  },
  plugins: [
    definePlugin((compiler) => {
      const RuntimeGlobals = compiler.rspack.RuntimeGlobals;

      class MockNormalRuntimeModule extends RuntimeModule {
        constructor() {
          super('mock-normal', RuntimeModule.STAGE_NORMAL);
        }

        generate() {
          return `${RuntimeGlobals.require}.mockNormal = "normal";`;
        }
      }

      class MockTriggerRuntimeModule extends RuntimeModule {
        constructor() {
          super('mock-trigger', RuntimeModule.STAGE_TRIGGER);
        }

        generate() {
          return `${RuntimeGlobals.require}.mockTrigger = "trigger";`;
        }
      }

      compiler.hooks.thisCompilation.tap('MockRuntimePlugin', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'MockRuntimePlugin',
          (chunk, set) => {
            set.add(RuntimeGlobals.publicPath);
            set.add(RuntimeGlobals.getChunkScriptFilename);
            compilation.addRuntimeModule(chunk, new MockTriggerRuntimeModule());
            compilation.addRuntimeModule(chunk, new MockNormalRuntimeModule());
          },
        );
      });
    }),
  ],
});
