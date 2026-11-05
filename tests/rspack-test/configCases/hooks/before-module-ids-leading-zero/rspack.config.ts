import { defineConfig } from '@rspack/cli';
import type { NormalModule, Compiler } from '@rspack/core';

// beforeModuleIds receives proxies with a string identifier and writable id.
type ModuleWithId = Omit<NormalModule, 'identifier'> & {
  identifier: string;
  id: string | number | null;
};

class LeadingZeroModuleIdPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'LeadingZeroModuleIdPlugin',
      (compilation) => {
        compilation.hooks.beforeModuleIds.tap(
          'LeadingZeroModuleIdPlugin',
          (modules) => {
            for (const module of modules as unknown as Iterable<ModuleWithId>) {
              if (module.resource?.endsWith('lazy-a.js')) {
                module.id = '0795';
              }
            }
          },
        );
      },
    );
  }
}

export default defineConfig({
  target: 'node',
  mode: 'production',
  optimization: {
    minimize: false,
  },
  plugins: [new LeadingZeroModuleIdPlugin()],
});
