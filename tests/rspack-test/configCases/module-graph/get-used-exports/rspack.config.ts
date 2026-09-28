import { join, normalize } from 'node:path';
import { defineConfig } from '@rspack/cli';
import { NormalModule, type Compiler } from '@rspack/core';

const PLUGIN_NAME = 'Test';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.optimizeModules.tap(PLUGIN_NAME, () => {
        const moduleA = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            normalize(module.resource) ===
              normalize(join(import.meta.dirname, 'a.js')),
        );
        expect(
          compilation.moduleGraph.getUsedExports(moduleA!, 'main'),
        ).toEqual(['good']);
        expect(
          compilation.moduleGraph.getUsedExports(moduleA!, ['main']),
        ).toEqual(['good']);

        const moduleB = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            normalize(module.resource) ===
              normalize(join(import.meta.dirname, 'b.js')),
        );
        expect(compilation.moduleGraph.getUsedExports(moduleB!, 'main')).toBe(
          false,
        );
      });
    });
  }
}

export default defineConfig({
  target: 'web',
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    usedExports: true,
  },
  plugins: [new Plugin()],
});
