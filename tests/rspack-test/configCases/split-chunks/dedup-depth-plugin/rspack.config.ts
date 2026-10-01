import assert from 'node:assert/strict';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { experiments, optimize } from '@rspack/core';
import { modules, sharedSize } from '../intersections-min-size/modules.ts';

const { SplitChunksPlugin } = optimize;

const context = path.resolve(import.meta.dirname, '../intersections-min-size');

export default [
  { mode: 'production' as const, defaultDepth: 1 },
  { mode: 'development' as const, defaultDepth: 0 },
  { mode: 'none' as const, defaultDepth: 0 },
  { mode: undefined, defaultDepth: 1 },
]
  .flatMap(({ mode, defaultDepth }) =>
    [undefined, 0, 1, 2].map((dedupDepth) => ({
      mode,
      dedupDepth,
      expectedDepth: dedupDepth ?? defaultDepth,
    })),
  )
  .map(({ mode, dedupDepth, expectedDepth }, index) =>
    defineConfig({
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
        new experiments.VirtualModulesPlugin(modules),
        new SplitChunksPlugin({
          chunks: 'all',
          dedupDepth,
          minSize: sharedSize,
          maxInitialRequests: Infinity,
          maxAsyncRequests: Infinity,
          cacheGroups: { shared: { test: /[\\/]m[012]\.js$/, minChunks: 2 } },
        }),
        definePlugin({
          apply(compiler) {
            compiler.hooks.done.tap('AssertPluginDedupDepth', (stats) => {
              const { modules } = stats.toJson({
                all: false,
                modules: true,
                ids: true,
                groupModulesByType: false,
                groupModulesByPath: false,
              });
              assert.ok(modules);
              const shared = modules.filter(
                (module) =>
                  module.name !== undefined &&
                  /^\.\/m[012]\.js$/.test(module.name),
              );
              assert.equal(shared.length, 3);
              const sharedChunks = shared.map((module) => {
                assert.ok(module.chunks);
                return module.chunks;
              });
              for (const chunks of sharedChunks) {
                assert.equal(chunks.length, expectedDepth > 0 ? 2 : 3);
              }
              const common = sharedChunks[0].filter((chunk) =>
                sharedChunks.every((chunks) => chunks.includes(chunk)),
              );
              assert.equal(common.length, expectedDepth > 0 ? 1 : 2);
            });
          },
        }),
      ],
    }),
  );
