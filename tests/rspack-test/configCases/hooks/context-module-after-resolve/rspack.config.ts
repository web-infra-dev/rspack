import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.contextModuleFactory.tap(
      pluginName,
      (contextModuleFactory) => {
        contextModuleFactory.hooks.afterResolve.tap(
          pluginName,
          (resolveData) => {
            if (resolveData === false) return false;
            if (resolveData.request.includes('./dir')) {
              return false;
            }
          },
        );
      },
    );
  }
}

export default defineConfig({
  context: import.meta.dirname,
  entry: './index.js',
  module: {
    rules: [],
  },
  plugins: [new Plugin()],
});
