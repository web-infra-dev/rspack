import { defineConfig, definePlugin } from '@rspack/cli';
import type { Chunk } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  target: 'node',
  entry: {
    r1: './r1',
    r2: './r2',
    r3: './r3',
    r4: './r4',
    r5: './r5',
    r6: './r6',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      // Both groups share one priority.
      cacheGroups: {
        default: false,
        defaultVendors: false,
        // x is in all six entries, so this group has the most chunks and wins first.
        shared: {
          test: /[\\/]x\.js$/,
          name: 'shared',
        },
        // y comes from r1 and r2 and z from r3 and r4, so no chunk holds every module of this
        // group. Losing x to `shared` must leave y and z in it, split from all four entries.
        routes: {
          test: /[\\/][xyz]\.js$/,
          chunks: (chunk: Chunk) => /^r[1-4]$/.test(chunk.name ?? ''),
          minChunks: 2,
          name: 'routes',
        },
      },
    },
  },
  plugins: [
    definePlugin(function () {
      this.hooks.compilation.tap(
        'NamedResidualDisjointChunks',
        (compilation) => {
          compilation.hooks.afterSeal.tap('NamedResidualDisjointChunks', () => {
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
            const chunksWith = (file: string) =>
              Array.from(compilation.chunks)
                .map(files)
                .filter((chunkFiles) => chunkFiles.includes(file));
            const routes = Array.from(compilation.chunks).find(
              (chunk) => chunk.name === 'routes',
            );
            expect(routes).toBeDefined();
            expect(files(routes!)).toEqual(['y.js', 'z.js']);
            expect(chunksWith('y.js')).toEqual([['y.js', 'z.js']]);
            expect(chunksWith('z.js')).toEqual([['y.js', 'z.js']]);
            expect(chunksWith('x.js')).toEqual([['x.js']]);
          });
        },
      );
    }),
  ],
});
