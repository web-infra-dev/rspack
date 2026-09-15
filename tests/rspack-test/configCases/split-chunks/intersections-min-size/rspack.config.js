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
  { mode: 'development', usedExports: false },
  { mode: 'development', usedExports: true },
  { mode: 'development', usedExports: false, dedupDepth: 1 },
  { mode: 'development', usedExports: true, dedupDepth: 1 },
].map(
  (
    {
      mode = 'production',
      usedExports,
      dedupDepth,
      belowThreshold,
      higherOrder,
      singleton,
      filtered,
    },
    index,
  ) => ({
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
          ? (chunk) => !['c', 'd', 'e'].includes(chunk.name)
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
      {
        apply(compiler) {
          compiler.hooks.beforeRun.tap('AssertDedupDepthDefault', () => {
            assert.equal(
              compiler.options.optimization.splitChunks.dedupDepth,
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
            const shared = json.modules.filter((module) =>
              /^\.\/m[012]\.js$/.test(module.name),
            );
            assert.equal(shared.length, 3);
            // In the higher-order case, each first-round pair fails minSize.
            // Its descendants must still be explored to find {a,b} in round 2.
            const depth = dedupDepth ?? (mode === 'production' ? 1 : 0);
            const extracted =
              depth > 0 && !belowThreshold && (!higherOrder || depth >= 2);
            for (const module of shared) {
              assert.equal(
                module.chunks.length,
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
