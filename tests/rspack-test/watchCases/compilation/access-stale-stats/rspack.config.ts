import type { Stats } from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

const DEFAULT_REQUIRED_KEYS = [
  'hash',
  'version',
  'rspackVersion',
  'time',
  'builtAt',
  'publicPath',
  'outputPath',
  'errors',
  'errorsCount',
  'warnings',
  'warningsCount',
  'children',
];

const ALL_REQUIRED_KEYS = [
  'hash',
  'version',
  'rspackVersion',
  'env',
  'time',
  'builtAt',
  'publicPath',
  'outputPath',
  'assetsByChunkName',
  'assets',
  'filteredAssets',
  'chunks',
  'modules',
  'filteredModules',
  'entrypoints',
  'namedChunkGroups',
  'errors',
  'errorsCount',
  'warnings',
  'warningsCount',
  'children',
];

function expectKeysToContain(actualKeys: string[], requiredKeys: string[]) {
  requiredKeys.forEach((k) => {
    expect(actualKeys).toContain(k);
  });
}

let staleStats: Stats | null = null;

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.make.tap('PLUGIN', () => {
          const previousStats = staleStats;
          if (previousStats) {
            // Stats can be accessed at any time. Use `setTimeout` to simulate a moment when
            // Rust-side ModuleGraph is not accessible anymore. Accessing an "outdated" stats
            // object should not panic; it should still return a valid JSON shape.
            setTimeout(() => {
              const defaultKeys = Object.keys(previousStats.toJson());
              expectKeysToContain(defaultKeys, DEFAULT_REQUIRED_KEYS);

              const allKeys = Object.keys(previousStats.toJson({ all: true }));
              expectKeysToContain(allKeys, ALL_REQUIRED_KEYS);
            });
          }
        });

        compiler.hooks.done.tap('PLUGIN', (stats) => {
          // Capture the first stats instance, and access it later after it becomes "outdated".
          if (!staleStats) staleStats = stats;
        });
      },
    }),
  ],
});
