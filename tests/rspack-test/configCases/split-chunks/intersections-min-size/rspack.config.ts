import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { experiments } from '@rspack/core';
import { modules, sharedSize } from './modules.ts';

export default [
  { usedExports: false },
  { usedExports: true },
  { usedExports: false, dedupDepth: 0 },
  { usedExports: true, dedupDepth: 0 },
  { usedExports: false, dedupDepth: 1 },
  { usedExports: true, dedupDepth: 1 },
  { usedExports: true, dedupDepth: 1, belowThreshold: true },
  { usedExports: false, dedupDepth: 1, higherOrder: true },
  { usedExports: true, dedupDepth: 1, higherOrder: true },
  { usedExports: false, dedupDepth: 2, higherOrder: true },
  { usedExports: true, dedupDepth: 2, higherOrder: true },
  { usedExports: false, dedupDepth: 3, higherOrder: true },
  { usedExports: true, dedupDepth: 3, higherOrder: true },
  { usedExports: false, dedupDepth: 1, singleton: true },
  { usedExports: false, dedupDepth: 1, filtered: true },
  { usedExports: true, dedupDepth: 1, filtered: true },
  { mode: 'development' as const, usedExports: false },
  { mode: 'development' as const, usedExports: true },
  { mode: 'development' as const, usedExports: false, dedupDepth: 1 },
  { mode: 'development' as const, usedExports: true, dedupDepth: 1 },
].map(
  (
    {
      mode = 'production' as const,
      usedExports,
      dedupDepth,
      belowThreshold,
      higherOrder,
      singleton,
      filtered,
    },
    index,
  ) =>
    defineConfig({
      mode,
      target: 'node',
      entry: {
        a: './a',
        ...(singleton ? {} : { b: './b' }),
        ...(higherOrder || filtered
          ? { c: './pair01', d: './pair02', e: './pair12' }
          : { c: './c0', d: './c1', e: './c2' }),
        ...(filtered ? { p0: './c0', p1: './c1', p2: './c2' } : {}),
      },
      output: {
        filename: `[name]-${index}.js`,
        chunkFilename: `[name]-${index}.js`,
      },
      optimization: {
        minimize: false,
        concatenateModules: false,
        splitChunks: {
          chunks: filtered
            ? (chunk) =>
                chunk.name === undefined ||
                !['c', 'd', 'e'].includes(chunk.name)
            : 'all',
          usedExports,
          ...(dedupDepth === undefined ? {} : { dedupDepth }),
          ...(singleton ? { minChunks: 1 } : {}),
          minSize: sharedSize + (belowThreshold ? 1 : 0),
          minSizeReduction: 0,
          maxInitialRequests: Infinity,
          maxAsyncRequests: Infinity,
          cacheGroups: { defaultVendors: false },
        },
      },
      plugins: [
        new experiments.VirtualModulesPlugin(modules),
        definePlugin({
          apply(compiler) {
            compiler.hooks.beforeRun.tap('AssertDedupDepthDefault', () => {
              const { splitChunks } = compiler.options.optimization;
              assert.ok(splitChunks);
              assert.equal(
                splitChunks.dedupDepth,
                dedupDepth ?? (mode === 'production' ? 1 : 0),
              );
            });
            compiler.hooks.done.tap('AssertIntersectionCandidates', (stats) => {
              const json = stats.toJson({
                all: false,
                modules: true,
                ids: true,
                groupModulesByType: false,
                groupModulesByPath: false,
              });
              assert.ok(json.modules);
              const shared = json.modules.filter(
                (module) =>
                  module.name !== undefined &&
                  /^\.\/m[012]\.js$/.test(module.name),
              );
              assert.equal(shared.length, 3);
              const sharedChunks = shared.map((module) => {
                assert.ok(module.chunks);
                return module.chunks;
              });
              // In the higher-order case, each first-round pair fails minSize.
              // Its descendants must still be explored to find {a,b} in round 2.
              const depth = dedupDepth ?? (mode === 'production' ? 1 : 0);
              const extracted =
                depth > 0 && !belowThreshold && (!higherOrder || depth >= 2);
              for (const chunks of sharedChunks) {
                assert.equal(
                  chunks.length,
                  filtered
                    ? 4
                    : higherOrder
                      ? extracted
                        ? 3
                        : 4
                      : extracted || singleton
                        ? 2
                        : 3,
                );
              }
              const common = sharedChunks[0].filter((chunk) =>
                sharedChunks.every((chunks) => chunks.includes(chunk)),
              );
              assert.equal(common.length, extracted || singleton ? 1 : 2);
              // Each raw pairwise intersection in the filtered case contains
              // only two modules and fails minSize. Filtering c/d/e maps them
              // all to {a,b}, where the combined three modules pass minSize.
            });
          },
        }),
      ],
    }),
);
