import { defineConfig } from '@rspack/cli';
import type { Chunk } from '@rspack/core';

let counters = [0, 0];
let seen = [new Set<Chunk>(), new Set<Chunk>()];
let compilationIndex = -1;

export default defineConfig({
  mode: 'development',
  target: 'node',
  entry: './index.js',
  devtool: false,
  output: { filename: '[name].js', chunkFilename: '[name].js' },
  optimization: {
    minimize: false,
    concatenateModules: false,
    usedExports: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        ...Object.fromEntries(
          [0, 1].map((group) => [
            `group${group}`,
            {
              test: /shared-\d+\.js$/,
              name: `shared-${group}`,
              priority: 0,
              minSize: 0,
              minChunks: 2,
              enforce: true,
              chunks(chunk: Chunk) {
                counters[group]++;
                seen[group].add(chunk);
                return group === 0 || !chunk.canBeInitial();
              },
            },
          ]),
        ),
      },
    },
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('AssertSelectorMemo', () => {
          compilationIndex++;
          counters = [0, 0];
          seen = [new Set<Chunk>(), new Set<Chunk>()];
        });
        compiler.hooks.done.tap('AssertSelectorMemo', () => {
          for (let group = 0; group < 2; group++) {
            expect(seen[group].size).toBe(3);
            expect(counters[group]).toBeLessThanOrEqual(
              compilationIndex < 2 ? 3 : 5,
            );
            if (compilationIndex === 2)
              expect(counters[group]).toBeGreaterThan(3);
          }
        });
      },
    },
  ],
});
