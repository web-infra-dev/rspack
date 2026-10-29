import { defineConfig, definePlugin } from '@rspack/cli';

/** @typedef {import("@rspack/core").Compiler} Compiler */

export default defineConfig({
  target: 'node',
  plugins: [
    /** @this {Compiler} */
    definePlugin(function () {
      this.hooks.compilation.tap('NamedChunksAfterRemove', (compilation) => {
        compilation.hooks.afterSeal.tap('NamedChunksAfterRemove', () => {
          // The "empty" chunk only has a module that is already in the parent chunk,
          // so RemoveEmptyChunksPlugin removes it together with its name.
          expect(Array.from(compilation.namedChunks.keys())).not.toContain(
            'empty',
          );
          for (const name of compilation.namedChunks.keys()) {
            expect(compilation.namedChunks.get(name)).toBeDefined();
          }
        });
      });
    }),
  ],
});
