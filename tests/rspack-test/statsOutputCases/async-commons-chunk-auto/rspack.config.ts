import type { Module, StatsOptions } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const stats: StatsOptions = {
  hash: false,
  timings: false,
  builtAt: false,
  assets: false,
  chunks: true,
  chunkOrigins: true,
  modules: false,
  entrypoints: 'auto',
};
export default defineConfig([
  {
    name: 'disabled',
    mode: 'production',
    entry: {
      main: './',
      a: './a',
      b: './b',
      c: './c',
    },
    output: {
      filename: 'disabled/[name].js',
    },
    optimization: {
      splitChunks: false,
    },
    stats,
  },
  {
    name: 'default',
    mode: 'production',
    entry: {
      main: './',
      a: './a',
      b: './b',
      c: './c',
    },
    output: {
      filename: 'default/[name].js',
    },
    optimization: {
      splitChunks: {
        minSize: 0, // enforce all
      },
    },
    stats,
  },

  {
    name: 'vendors',
    mode: 'production',
    entry: {
      main: './',
      a: './a',
      b: './b',
      c: './c',
    },
    output: {
      filename: 'vendors/[name].js',
    },
    optimization: {
      splitChunks: {
        cacheGroups: {
          vendors: {
            test: /[\\/]node_modules[\\/]/,
            chunks: 'initial',
            name: 'vendors',
            enforce: true,
          },
        },
      },
    },
    stats,
  },

  {
    name: 'multiple-vendors',
    mode: 'production',
    entry: {
      main: './',
      a: './a',
      b: './b',
      c: './c',
    },
    output: {
      filename: 'multiple-vendors/[name].js',
    },
    optimization: {
      splitChunks: {
        minSize: 0, // enforce all
        chunks: 'all',
        cacheGroups: {
          // @ts-expect-error Rspack does not support function cache groups yet.
          libs: (module: Module) => {
            const name = module.nameForCondition();
            if (!name) return;
            const match = /[\\/](xyz|x)\.js/.exec(name);
            if (match)
              return {
                name: 'libs-' + match[1],
                enforce: true,
              };
          },
          // @ts-expect-error Rspack does not support string cache groups yet.
          vendors: path.resolve(import.meta.dirname, 'node_modules'),
        },
      },
    },
    stats,
  },
  {
    name: 'all',
    mode: 'production',
    entry: {
      main: './',
      a: './a',
      b: './b',
      c: './c',
    },
    output: {
      filename: 'all/[name].js',
    },
    optimization: {
      splitChunks: {
        minSize: 0, // enforce all
        chunks: 'all',
        cacheGroups: {
          // @ts-expect-error Rspack does not support string cache groups yet.
          vendors: path.resolve(import.meta.dirname, 'node_modules'),
        },
      },
    },
    stats,
  },
]);
