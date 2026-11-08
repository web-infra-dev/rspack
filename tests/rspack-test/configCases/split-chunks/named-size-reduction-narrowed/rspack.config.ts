import { defineConfig, definePlugin } from '@rspack/cli';
import type { Chunk } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  target: 'node',
  entry: {
    e1: './all',
    e2: './all',
    e3: './without-q',
    e4: './only-m',
    e5: './only-m',
    e6: './only-m',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  module: {
    rules: [
      { test: /[\\/]m\.js$/, loader: './size-loader', options: { size: 100 } },
      { test: /[\\/]n\.js$/, loader: './size-loader', options: { size: 100 } },
      { test: /[\\/]q\.js$/, loader: './size-loader', options: { size: 200 } },
    ],
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      minChunks: 2,
      // All groups share one priority.
      cacheGroups: {
        default: false,
        defaultVendors: false,
        // Selects m and n from e1, e2 and e3.
        g: {
          test: /[\\/][mn]\.js$/,
          chunks: /^e[1-3]$/,
          name: 'g',
        },
        // Selects n and q from e1 and e2, saving (100 + 200) * 1 = 300.
        h: {
          test: /[\\/][nq]\.js$/,
          chunks: /^e[12]$/,
        },
        // Selects the most chunks, so it wins first and moves m out of e3. That narrows g to
        // e1 and e2 without removing a module: g now saves (100 + 100) * 1 = 200 rather than
        // (100 + 100) * 2, and must lose n to h.
        w: {
          test: /[\\/]m\.js$/,
          chunks: /^e[3-6]$/,
        },
      },
    },
  },
  plugins: [
    definePlugin(function () {
      this.hooks.compilation.tap(
        'NamedSizeReductionNarrowed',
        (compilation) => {
          compilation.hooks.afterSeal.tap('NamedSizeReductionNarrowed', () => {
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
            const g = compilation.namedChunks.get('g');
            expect(g).toBeDefined();
            expect(files(g!)).toEqual(['m.js']);
            expect(chunksWith('q.js')).toEqual([['n.js', 'q.js']]);
            // One copy of m for e1 and e2 (g), the other for e3 to e6 (w).
            expect(chunksWith('m.js')).toEqual([['m.js'], ['m.js']]);
          });
        },
      );
    }),
  ],
});
