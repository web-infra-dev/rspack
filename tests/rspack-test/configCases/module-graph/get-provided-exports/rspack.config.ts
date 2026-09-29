import { defineConfig } from '@rspack/cli';
import { type Compiler, NormalModule } from '@rspack/core';
import { join, normalize } from 'node:path';

const PLUGIN_NAME = 'Test';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      compilation.hooks.optimizeModules.tap(PLUGIN_NAME, () => {
        // ESM module with named exports
        const moduleA = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            normalize(module.resource) ===
              normalize(join(import.meta.dirname, 'a.js')),
        )!;
        expect(
          (
            compilation.moduleGraph.getProvidedExports(moduleA) as string[]
          ).sort(),
        ).toEqual(['bar', 'foo']);

        // ESM module with no exports (side-effect only)
        const moduleB = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            normalize(module.resource) ===
              normalize(join(import.meta.dirname, 'b.js')),
        )!;
        expect(compilation.moduleGraph.getProvidedExports(moduleB)).toEqual([]);

        // ESM module with default export
        const moduleD = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            normalize(module.resource) ===
              normalize(join(import.meta.dirname, 'd.js')),
        )!;
        expect(compilation.moduleGraph.getProvidedExports(moduleD)).toEqual([
          'default',
        ]);

        // CJS module (all exports potentially provided)
        const moduleC = Array.from(compilation.modules).find(
          (module) =>
            module instanceof NormalModule &&
            normalize(module.resource) ===
              normalize(join(import.meta.dirname, 'c.js')),
        )!;
        expect(compilation.moduleGraph.getProvidedExports(moduleC)).toBe(true);
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
    providedExports: true,
  },
  plugins: [new Plugin()],
});
