import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

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
          test: /[\\/]a\.js$/,
          chunks: 'async',
          minSize: 0,
          enforce: true,
        },
      },
    },
  },
  plugins: [
    // Merges the "named" async chunk into the other async chunk.
    new rspack.optimize.LimitChunkCountPlugin({ maxChunks: 2 }),
    /** @this {Compiler} */
    definePlugin(function () {
      this.hooks.compilation.tap('NamedChunksAfterLimit', (compilation) => {
        compilation.hooks.afterSeal.tap('NamedChunksAfterLimit', () => {
          for (const name of compilation.namedChunks.keys()) {
            expect(compilation.namedChunks.get(name)).toBeDefined();
          }
          expect(compilation.namedChunks.get('named')?.name).toBe('named');
        });
      });
    }),
  ],
});
