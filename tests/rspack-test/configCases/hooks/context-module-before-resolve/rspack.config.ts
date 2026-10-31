import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.contextModuleFactory.tap(
      pluginName,
      (contextModuleFactory) => {
        contextModuleFactory.hooks.beforeResolve.tap(
          pluginName,
          (resolveData) => {
            if (!resolveData) return resolveData;
            if (resolveData.request.includes('./locale')) {
              resolveData.regExp = /[/\\](en(\.js)?|zh(\.js)?)$/;
              return resolveData;
            }
            // @ts-expect-error Rspack does not expose context dependencies in this hook yet.
            for (const d of resolveData.dependencies) {
              if (d.critical) d.critical = false;
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
