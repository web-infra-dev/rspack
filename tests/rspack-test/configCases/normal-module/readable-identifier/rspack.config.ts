import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { NormalModule, type Compiler } from '@rspack/core';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.finishMake.tap('PLUGIN', (compilation) => {
      for (const module of compilation.modules) {
        if (!(module instanceof NormalModule)) continue;
        if (module.resource === path.join(import.meta.dirname, 'bar.js')) {
          expect(module.readableIdentifier()).toBe('./bar.js');
        }
        if (
          module.resource ===
          path.join(import.meta.dirname, 'node_modules/foo/index.js')
        ) {
          expect(module.readableIdentifier()).toBe(
            './node_modules/foo/index.js',
          );
        }
      }
    });
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});
