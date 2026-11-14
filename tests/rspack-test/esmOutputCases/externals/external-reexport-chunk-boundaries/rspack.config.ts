import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  [
    'cycle',
    'acyclic',
    'interleaved',
    'rootless',
    'nested',
    'independent-entries',
    'array-stars',
  ].flatMap((variant) =>
    [false, true].map((minimize) => {
      const name = `${variant}-${minimize ? 'min' : 'readable'}`;
      const independentEntries = variant === 'independent-entries';
      const arrayStars = variant === 'array-stars';
      const shared =
        variant === 'interleaved' ||
        variant === 'rootless' ||
        independentEntries;
      return defineConfig({
        name,
        target: 'node',
        entry: independentEntries
          ? { first: `./${variant}/a.mjs`, second: `./${variant}/b.mjs` }
          : arrayStars
            ? ['./array-stars/first.mjs', './array-stars/second.mjs']
            : `./${variant}/index.js`,
        externalsType: 'module',
        externals: {
          '../ext.mjs': './ext.mjs',
          '../tail.mjs': './tail.mjs',
          '../side.mjs': './side.mjs',
          './shared.mjs': './shared.mjs',
          './y.mjs': './y.mjs',
          './z.mjs': './z.mjs',
        },
        output: {
          filename: `${name}/[name].mjs`,
          chunkFilename: `${name}/[name].mjs`,
        },
        plugins:
          independentEntries || arrayStars
            ? [new rspack.experiments.RslibPlugin()]
            : [],
        optimization: {
          minimize,
          runtimeChunk: false,
          splitChunks: {
            chunks: 'all',
            cacheGroups: {
              a: {
                test: shared ? /[\\/][ab]\.mjs$/ : /[\\/]a\.mjs$/,
                name: 'a',
                enforce: true,
              },
              b: shared
                ? false
                : {
                    test: /[\\/]b\.mjs$/,
                    name: 'b',
                    enforce: true,
                  },
              setup: {
                test: /[\\/]setup\.mjs$/,
                name: 'setup',
                enforce: true,
              },
            },
          },
        },
      });
    }),
  ),
);
