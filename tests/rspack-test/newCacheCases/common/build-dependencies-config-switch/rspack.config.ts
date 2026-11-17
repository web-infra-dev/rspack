import { defineConfig, definePlugin } from '@rspack/cli';
import type { PersistentCacheOptions } from '@rspack/core';
import path from 'node:path';

const steps: [string[], number][] = [
  [['a.json'], 1],
  [['a.json'], 0], // Unchanged
  [['b.json'], 1], // Replace
  [['a.json', 'b.json'], 1], // Add
  [['b.json', 'a.json', 'b.json'], 0], // Reorder and duplicate
  [['b.json'], 1], // Remove
  [[], 1], // Clear
  [[], 0],
];
let step = 0;

export default defineConfig({
  incremental: false,
  cache: { type: 'persistent' },
  plugins: [
    definePlugin({
      apply(compiler) {
        const [files, expectedBuilds] = steps[step++];
        const cache = compiler.options.cache as PersistentCacheOptions;
        cache.buildDependencies = files.map((file) =>
          path.join(import.meta.dirname, 'configs', file),
        );
        let builds = 0;
        compiler.hooks.compilation.tap(
          'BuildDependenciesConfigSwitch',
          (compilation) => {
            compilation.buildDependencies.add(
              path.join(import.meta.dirname, 'index.js'),
            );
            compilation.hooks.buildModule.tap(
              'BuildDependenciesConfigSwitch',
              () => {
                builds++;
              },
            );
          },
        );
        compiler.hooks.done.tap('BuildDependenciesConfigSwitch', () => {
          expect(builds).toBe(expectedBuilds);
        });
      },
    }),
  ],
});
