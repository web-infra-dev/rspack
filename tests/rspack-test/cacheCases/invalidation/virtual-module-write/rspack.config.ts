import { defineConfig, definePlugin } from '@rspack/cli';
import { experiments } from '@rspack/core';

let compilerIndex = 0;

export default defineConfig({
  mode: 'development',
  cache: { type: 'persistent' },
  plugins: [
    definePlugin((compiler) => {
      const virtualModules = new experiments.VirtualModulesPlugin({
        'virtual.js': 'module.exports = "first";',
      });
      virtualModules.apply(compiler);

      if (compilerIndex++ === 1) {
        compiler.hooks.thisCompilation.tap('WriteVirtualModule', () => {
          virtualModules.writeModule(
            'virtual.js',
            'module.exports = "second";',
          );
        });
      }
    }),
  ],
});
