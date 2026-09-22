import type { Module, Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

let moduleB: Module | undefined;

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Plugin', (compilation) => {
      compilation.hooks.processAssets.tap('Plugin', () => {
        const previousModule = moduleB;
        if (previousModule) {
          expect(() => {
            previousModule.size();
          }).toThrow(
            /Unable to access module with id =.* now. The module have been removed on the Rust side./,
          );
        }
        moduleB = Array.from(compilation.modules).find((module) =>
          module.identifier().includes('b.js'),
        );
      });
    });
  }
}

export default defineConfig({
  plugins: [new Plugin()],
});
