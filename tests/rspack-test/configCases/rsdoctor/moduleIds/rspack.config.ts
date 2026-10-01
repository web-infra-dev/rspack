import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RsdoctorPlugin },
} = rspack;

export default defineConfig({
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new RsdoctorPlugin({
      moduleGraphFeatures: ['graph', 'ids'],
      chunkGraphFeatures: false,
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'TestPlugin::ModuleIds',
          (compilation) => {
            const hooks = RsdoctorPlugin.getCompilationHooks(compilation);
            hooks.moduleIds.tap('TestPlugin::ModuleIds', (data) => {
              const { moduleIds } = data;
              expect(moduleIds.length).toBe(2);
            });
          },
        );
      },
    }),
  ],
});
