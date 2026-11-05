import { defineConfig, definePlugin } from '@rspack/cli';

/** @typedef {import("@rspack/core").Compiler} Compiler */

export default defineConfig({
  target: 'node',
  optimization: {
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        // Moves the only module of the merged "first" and "second" chunk to "shared",
        // so RemoveEmptyChunksPlugin removes the chunk that holds both names.
        shared: {
          test: /x\.js$/,
          name: 'shared',
          chunks: 'async',
          minSize: 0,
          enforce: true,
          reuseExistingChunk: false,
        },
      },
    },
  },
  plugins: [
    /** @this {Compiler} */
    definePlugin(function () {
      this.hooks.compilation.tap('NamedChunksAfterRemove', (compilation) => {
        compilation.hooks.afterSeal.tap('NamedChunksAfterRemove', () => {
          const chunks = new Set(compilation.chunks);
          const stale = Array.from(compilation.namedChunks)
            .filter(([, chunk]) => !chunks.has(chunk))
            .map(([name]) => name);
          expect(stale).toEqual([]);
        });
      });
    }),
  ],
});
