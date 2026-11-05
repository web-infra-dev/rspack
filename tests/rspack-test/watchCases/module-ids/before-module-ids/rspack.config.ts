import { defineConfig } from '@rspack/cli';
import type { NormalModule, Compiler } from '@rspack/core';

// beforeModuleIds receives proxies with a string identifier and writable id.
type ModuleWithId = Omit<NormalModule, 'identifier'> & {
  identifier: string;
  id: string | number | null;
};

class CustomModuleIdPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('CustomModuleIdPlugin', (compilation) => {
      compilation.hooks.beforeModuleIds.tap(
        'CustomModuleIdPlugin',
        (modules) => {
          for (const module of modules as unknown as Iterable<ModuleWithId>) {
            if (module.identifier.includes('index.js')) {
              module.id = 'custom-entry-id';
            }
          }
        },
      );
    });
  }
}

export default defineConfig({
  cache: { type: 'memory' },
  optimization: {
    moduleIds: 'deterministic',
    concatenateModules: false,
    inlineExports: false,
  },
  plugins: [new CustomModuleIdPlugin()],
  incremental: true,
});
