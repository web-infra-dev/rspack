const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const sharedSize = [0, 1, 2].reduce(
  (sum, index) => sum + fs.statSync(path.join(__dirname, `m${index}.js`)).size,
  0,
);

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  { usedExports: false },
  { usedExports: true },
  { usedExports: false, optimizeForSize: false },
  { usedExports: true, optimizeForSize: false },
  { usedExports: false, optimizeForSize: true },
  { usedExports: true, optimizeForSize: true },
  { usedExports: true, optimizeForSize: true, belowThreshold: true },
  { usedExports: false, optimizeForSize: true, higherOrder: true },
  { usedExports: true, optimizeForSize: true, higherOrder: true },
  { usedExports: false, optimizeForSize: true, singleton: true },
  { usedExports: false, optimizeForSize: true, filtered: true },
  { usedExports: true, optimizeForSize: true, filtered: true },
].map(
  (
    {
      usedExports,
      optimizeForSize,
      belowThreshold,
      higherOrder,
      singleton,
      filtered,
    },
    index,
  ) => ({
    mode: 'production',
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
          ? (chunk) => !['c', 'd', 'e'].includes(chunk.name)
          : 'all',
        usedExports,
        ...(optimizeForSize === undefined ? {} : { optimizeForSize }),
        ...(singleton ? { minChunks: 1 } : {}),
        minSize: sharedSize + (belowThreshold ? 1 : 0),
        minSizeReduction: 0,
        maxInitialRequests: Infinity,
        maxAsyncRequests: Infinity,
        cacheGroups: { defaultVendors: false },
      },
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.beforeRun.tap('AssertOptimizeForSizeDefault', () => {
            assert.equal(
              compiler.options.optimization.splitChunks.optimizeForSize,
              optimizeForSize ?? false,
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
            const shared = json.modules.filter((module) =>
              /^\.\/m[012]\.js$/.test(module.name),
            );
            assert.equal(shared.length, 3);
            // No module has exactly the target {a,b} (or singleton {a}) chunk
            // set. Only opt-in pairwise discovery can create that candidate.
            // In the higher-order case, each pair shares only two modules and
            // fails minSize. Generated candidates do not re-enter intersections,
            // so the three-way intersection is deliberately not discovered.
            const extracted =
              optimizeForSize && !belowThreshold && !higherOrder;
            for (const module of shared) {
              assert.equal(
                module.chunks.length,
                filtered || higherOrder ? 4 : extracted || singleton ? 2 : 3,
              );
            }
            const common = shared[0].chunks.filter((chunk) =>
              shared.every((module) => module.chunks.includes(chunk)),
            );
            assert.equal(common.length, extracted || singleton ? 1 : 2);
            // Each raw pairwise intersection in the filtered case contains
            // only two modules and fails minSize. Filtering c/d/e maps them
            // all to {a,b}, where the combined three modules pass minSize.
          });
        },
      },
    ],
  }),
);
