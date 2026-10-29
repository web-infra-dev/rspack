import { defineConfig, definePlugin } from '@rspack/cli';

/** @typedef {import("@rspack/core").Compiler} Compiler */

export default defineConfig({
  target: 'node',
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        named: {
          name: 'named',
          chunks: 'async',
          minSize: 0,
        },
      },
    },
  },
  plugins: [
    /** @this {Compiler} */
    definePlugin(function () {
      this.hooks.compilation.tap('NamedChunksAfterMerge', (compilation) => {
        compilation.hooks.afterSeal.tap('NamedChunksAfterMerge', () => {
          // MergeDuplicateChunksPlugin removes one of the two async chunks.
          // Every name must still refer to a chunk of the compilation.
          for (const name of compilation.namedChunks.keys()) {
            expect(compilation.namedChunks.get(name)).toBeDefined();
          }
          expect(compilation.namedChunks.get('named')?.name).toBe('named');
        });
      });
    }),
  ],
});
