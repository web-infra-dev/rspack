import { defineConfig, definePlugin } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

type Variant =
  | 'normal'
  | 'callback'
  | 'pruned-bootstrap'
  | 'pruned-helper'
  | 'protected-bootstrap'
  | 'protected-same-priority';

function createConfig(
  index: number,
  dedupDepth: number,
  concatenateModules: boolean,
  variant: Variant = 'normal',
): Configuration {
  const prunedHelper = variant === 'pruned-helper';
  const prunedBootstrap = variant === 'pruned-bootstrap';
  const protectedSamePriority = variant === 'protected-same-priority';
  const protectedBootstrap =
    variant === 'protected-bootstrap' || protectedSamePriority;
  return {
    mode: 'production',
    target: 'web',
    entry: {
      bootstrap: './bootstrap',
      page1: prunedHelper ? './track-only' : './page',
      page2: prunedHelper ? './track-only' : './page',
      ...(prunedHelper ? { extra1: './extra', extra2: './extra' } : {}),
      // Four selected chunks make the protected group win before common at the same priority.
      ...(protectedSamePriority
        ? {
            protected1: './bootstrap',
            protected2: './bootstrap',
            protected3: './bootstrap',
          }
        : {}),
    },
    output: {
      filename: `[name]-${index}.js`,
    },
    optimization: {
      concatenateModules,
      splitChunks: {
        dedupDepth,
        minSize: 0,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          prelude: {
            test: /[\\/]prelude\.js$/,
            chunks: prunedHelper ? /^extra/ : /^bootstrap$/,
            name: 'prelude',
            minChunks: 1,
            priority: 100,
          },
          protected: protectedBootstrap && {
            test: /[\\/]singleton\.js$/,
            chunks: protectedSamePriority
              ? /^(bootstrap|protected)/
              : /^bootstrap$/,
            name: 'bootstrap',
            minChunks: 1,
            priority: protectedSamePriority ? 0 : 50,
          },
          common: {
            test: /[\\/]modules[\\/]/,
            chunks: protectedSamePriority ? /^(bootstrap|page)/ : 'all',
            name: variant === 'callback' ? () => 'common' : 'common',
            minChunks: 2,
            maxInitialRequests: prunedHelper || prunedBootstrap ? 2 : Infinity,
          },
        },
      },
    },
    plugins: [
      definePlugin(function () {
        this.hooks.compilation.tap('CheckNamedSources', (compilation) => {
          // Check before concatenation so scope hoisting cannot hide a duplicate.
          compilation.hooks.optimizeTree.tap('CheckNamedSources', (chunks) => {
            const placements = (file: string) =>
              Array.from(chunks)
                .filter((chunk) =>
                  Array.from(
                    compilation.chunkGraph.getChunkModulesIterable(chunk),
                  ).some((module) =>
                    module
                      .identifier()
                      .replaceAll('\\', '/')
                      .endsWith(`/${file}.js`),
                  ),
                )
                .map((chunk) => chunk.name)
                .sort();
            expect(placements('singleton')).toEqual(
              prunedHelper
                ? ['bootstrap', 'common', 'extra1', 'extra2']
                : prunedBootstrap || protectedBootstrap
                  ? ['bootstrap', 'common']
                  : ['common'],
            );
            expect(placements('helper')).toEqual(
              protectedSamePriority
                ? ['common', 'protected1', 'protected2', 'protected3']
                : prunedHelper
                  ? ['bootstrap', 'extra1', 'extra2']
                  : prunedBootstrap
                    ? ['bootstrap', 'common']
                    : ['common'],
            );
          });
        });
      }),
    ],
  };
}

export default defineConfig([
  createConfig(0, 0, false),
  createConfig(1, 0, true),
  createConfig(2, 3, false),
  createConfig(3, 3, true),
  createConfig(4, 1, true, 'callback'),
  createConfig(5, 1, false, 'pruned-bootstrap'),
  createConfig(6, 1, false, 'pruned-helper'),
  createConfig(7, 1, false, 'protected-bootstrap'),
  createConfig(8, 1, false, 'protected-same-priority'),
]);
