import { defineConfig, definePlugin } from '@rspack/cli';
import type { Chunk } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  target: 'node',
  entry: {
    e1: './with-y',
    e2: './with-y',
    e3: './without-y',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  module: {
    rules: [
      { test: /[\\/]x\.js$/, loader: './size-loader', options: { size: 50 } },
      { test: /[\\/]y\.js$/, loader: './size-loader', options: { size: 1000 } },
      { test: /[\\/]z\.js$/, loader: './size-loader', options: { size: 700 } },
    ],
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      minChunks: 2,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        // Both groups select e1, e2 and e3 at the same priority, so the size each split
        // saves decides which one takes x. y is not in e3, so the named group only saves
        // 50 * 2 + 1000 * 1 = 1100, less than the anonymous group's (50 + 700) * 2 = 1500.
        // Counting y in all three chunks would overstate it as (50 + 1000) * 2 = 2100.
        named: {
          test: /[\\/][xy]\.js$/,
          name: 'named',
        },
        anonymous: {
          test: /[\\/][xz]\.js$/,
        },
      },
    },
  },
  plugins: [
    definePlugin(function () {
      this.hooks.compilation.tap('NamedSizeReduction', (compilation) => {
        compilation.hooks.afterSeal.tap('NamedSizeReduction', () => {
          const files = (chunk: Chunk) =>
            Array.from(compilation.chunkGraph.getChunkModulesIterable(chunk))
              .map((module) =>
                module
                  .nameForCondition()
                  ?.replaceAll('\\', '/')
                  .split('/')
                  .pop(),
              )
              .sort();
          const named = compilation.namedChunks.get('named');
          expect(named).toBeDefined();
          expect(files(named!)).toEqual(['y.js']);
          expect(
            Array.from(compilation.chunks)
              .map(files)
              .filter((chunkFiles) => chunkFiles.includes('x.js')),
          ).toEqual([['x.js', 'z.js']]);

          // e3 no longer loads y through the named chunk.
          const chunkNames = (entry: string) =>
            compilation.entrypoints
              .get(entry)!
              .chunks.map((chunk) => chunk.name);
          expect(chunkNames('e1')).toContain('named');
          expect(chunkNames('e2')).toContain('named');
          expect(chunkNames('e3')).not.toContain('named');
        });
      });
    }),
  ],
});
