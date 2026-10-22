import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('Plugin', (compilation) => {
      compilation.hooks.processAssets.tap('Plugin', () => {
        const module = Array.from(compilation.modules).find(
          (module) => module.buildInfo.affected,
        );
        expect(!!module).toBe(true);
      });
    });
  }
}

export default defineConfig({
  plugins: [new Plugin()],
});
