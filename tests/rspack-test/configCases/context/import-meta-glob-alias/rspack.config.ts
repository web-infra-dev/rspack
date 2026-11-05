import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

const pluginName = 'RelocateGlobRoot';

class RelocateGlobRootPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.contextModuleFactory.tap(
      pluginName,
      (contextModuleFactory) => {
        contextModuleFactory.hooks.afterResolve.tap(
          pluginName,
          (resolveData) => {
            if (
              resolveData !== false &&
              resolveData.request.includes('?shared-directory-prefix')
            ) {
              expect(resolveData.resource).toBe(
                path.join(import.meta.dirname, 'src/glob-imports'),
              );
              expect(resolveData.recursive).toBe(false);
            }
            if (
              resolveData !== false &&
              resolveData.request.includes('@relocated')
            ) {
              resolveData.resource = path.join(
                import.meta.dirname,
                'src/[aliased]/{assets}',
              );
            }
          },
        );
      },
    );
  }
}

export default defineConfig({
  context: import.meta.dirname,
  resolve: {
    alias: {
      dir: [],
      ignored: false,
      nested$: path.join(import.meta.dirname, 'src', 'does-not-exist'),
      '@': path.join(import.meta.dirname, 'src'),
      '@meta': path.join(import.meta.dirname, 'src/[aliased]/{assets}'),
      '@relocated': path.join(import.meta.dirname, 'src/glob-imports'),
      '~': import.meta.dirname,
      siblingLeft: path.join(import.meta.dirname, 'src/sibling-left'),
      siblingRight: path.join(import.meta.dirname, 'src/sibling-right'),
      'special-dir': path.join(import.meta.dirname, 'src', '[dir]'),
    },
  },
  plugins: [new RelocateGlobRootPlugin()],
});
