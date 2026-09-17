import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
const sharedSize = Array.from(
  { length: 5 },
  (_, i) => fs.statSync(path.join(import.meta.dirname, `m${i}.js`)).size,
).reduce((a, b) => a + b, 0);

// Each module belongs to {a,b} and four of the five c entries. Discovering
// {a,b} requires intersecting five original sets: round 1 combines at most two,
// round 2 at most four, and round 3 can finally include all five. Intermediate
// intersections fail minSize, but their smaller descendants must survive.
/** @type {import("@rspack/core").Configuration[]} */
export default [false, true]
  .flatMap((usedExports) =>
    [0, 1, 2, 3, 4, 0xffffffff].map((dedupDepth) => ({
      usedExports,
      dedupDepth,
    })),
  )
  .map(({ usedExports, dedupDepth }, index) => ({
    mode: 'production',
    target: 'node',
    entry: {
      a: './a',
      b: './b',
      ...Object.fromEntries(
        Array.from({ length: 5 }, (_, i) => [`c${i}`, `./c${i}`]),
      ),
    },
    output: {
      filename: `[name]-${index}.js`,
      chunkFilename: `[name]-${index}.js`,
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      splitChunks: {
        chunks: 'all',
        usedExports,
        dedupDepth,
        minSize: sharedSize,
        minSizeReduction: 0,
        maxInitialRequests: Infinity,
        maxAsyncRequests: Infinity,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          shared: { test: /[\\/]m[0-4]\.js$/, minChunks: 2 },
        },
      },
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.done.tap('AssertDedupDepth', (stats) => {
            const { modules } = stats.toJson({
              all: false,
              modules: true,
              ids: true,
              groupModulesByType: false,
              groupModulesByPath: false,
            });
            const shared = modules.filter((module) =>
              /^\.\/m[0-4]\.js$/.test(module.name),
            );
            assert.equal(shared.length, 5);
            const extracted = dedupDepth >= 3;
            for (const module of shared)
              assert.equal(module.chunks.length, extracted ? 5 : 6);
            const common = shared[0].chunks.filter((chunk) =>
              shared.every((module) => module.chunks.includes(chunk)),
            );
            assert.equal(common.length, extracted ? 1 : 2);
          });
        },
      },
    ],
  }));
