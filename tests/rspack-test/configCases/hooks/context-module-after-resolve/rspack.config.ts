import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const pluginName = 'plugin';

class Plugin {
  apply(compiler: Compiler) {
    let checkRetainedDependency: (() => void) | undefined;
    compiler.hooks.done.tap(pluginName, () => {
      expect(checkRetainedDependency).toBeDefined();
      checkRetainedDependency?.();
    });
    compiler.hooks.contextModuleFactory.tap(
      pluginName,
      (contextModuleFactory) => {
        contextModuleFactory.hooks.afterResolve.tap(
          pluginName,
          (resolveData) => {
            if (resolveData === false) return false;
            if (resolveData.request.includes('./dir')) {
              const dependency = resolveData.dependencies[0];
              const type = dependency.type;
              dependency.critical = false;
              checkRetainedDependency = () => {
                expect(dependency.type).toBe(type);
                expect(dependency._parentModule).toBeNull();
                expect(dependency.ids).toBeUndefined();
                dependency.critical = false;
                expect(dependency.critical).toBe(false);
              };
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
