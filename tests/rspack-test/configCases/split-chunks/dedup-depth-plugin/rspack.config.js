const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { SplitChunksPlugin } = require('@rspack/core').optimize;

const context = path.resolve(__dirname, '../intersections-min-size');
const minSize = [0, 1, 2].reduce(
  (sum, index) => sum + fs.statSync(path.join(context, `m${index}.js`)).size,
  0,
);

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  { mode: 'production', defaultDepth: 1 },
  { mode: 'development', defaultDepth: 0 },
  { mode: 'none', defaultDepth: 0 },
  { mode: undefined, defaultDepth: 1 },
]
  .flatMap(({ mode, defaultDepth }) =>
    [undefined, 0, 1, 2].map((dedupDepth) => ({
      mode,
      dedupDepth,
      expectedDepth: dedupDepth ?? defaultDepth,
    })),
  )
  .map(({ mode, dedupDepth, expectedDepth }, index) => ({
    context,
    mode,
    target: 'node',
    entry: { a: './a', b: './b', c: './c0', d: './c1', e: './c2' },
    output: {
      filename: `[name]-${index}.js`,
      chunkFilename: `[name]-${index}.js`,
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      splitChunks: false,
    },
    plugins: [
      new SplitChunksPlugin({
        chunks: 'all',
        dedupDepth,
        minSize,
        maxInitialRequests: Infinity,
        maxAsyncRequests: Infinity,
        cacheGroups: { shared: { test: /[\\/]m[012]\.js$/, minChunks: 2 } },
      }),
      {
        apply(compiler) {
          compiler.hooks.done.tap('AssertPluginDedupDepth', (stats) => {
            const { modules } = stats.toJson({
              all: false,
              modules: true,
              ids: true,
              groupModulesByType: false,
              groupModulesByPath: false,
            });
            const shared = modules.filter((module) =>
              /^\.\/m[012]\.js$/.test(module.name),
            );
            assert.equal(shared.length, 3);
            for (const module of shared) {
              assert.equal(module.chunks.length, expectedDepth > 0 ? 2 : 3);
            }
            const common = shared[0].chunks.filter((chunk) =>
              shared.every((module) => module.chunks.includes(chunk)),
            );
            assert.equal(common.length, expectedDepth > 0 ? 1 : 2);
          });
        },
      },
    ],
  }));
