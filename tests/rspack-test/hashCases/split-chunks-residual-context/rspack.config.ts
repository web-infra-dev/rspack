import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { experiments } from '@rspack/core';

const test = /[\\/][ab]\.js$/;
const contexts = Array.from({ length: 8 }, (_, index) => `ctx${index}`);

export default defineConfig(
  contexts.map((name) => ({
    name,
    context: path.resolve('/', name),
    mode: 'production',
    devtool: false,
    entry: {
      e1: './entry',
      e2: './entry',
      e3: './entry',
      e4: './entry',
      e5: './entry',
    },
    output: {
      path: path.join(import.meta.dirname, 'dist', name),
      filename: '[name].[contenthash].js',
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      usedExports: false,
      splitChunks: {
        minSize: 0,
        minChunks: 2,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          named: { test, chunks: /^e[145]$/, name: 'named' },
          residual: { test, chunks: /^e[123]$/ },
        },
      },
    },
    plugins: [
      new experiments.VirtualModulesPlugin({
        'entry.js': "import './a'; import './b';",
        'a.js': 'globalThis.a = 1;',
        'b.js': "globalThis.b = 'a larger module';",
      }),
    ],
  })),
);
