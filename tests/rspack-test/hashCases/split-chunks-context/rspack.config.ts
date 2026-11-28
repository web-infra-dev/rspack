import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { experiments } from '@rspack/core';

const modules = {
  'main.js': "import('./lazy.js');",
  'lazy.js': "import './outer.js'; import './inner.js';",
  'outer.js': "import './shared-a.js'; import './shared-d.js';",
  'inner.js': "import './shared-b.js'; import './shared-c.js';",
  'shared-a.js': "globalThis.collect('a');",
  'shared-b.js': "globalThis.collect('b');",
  'shared-c.js': "globalThis.collect('c');",
  'shared-d.js': "globalThis.collect('d');",
};

export default defineConfig(
  ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'].map((name) => ({
    name,
    context: path.resolve('/rspack-split-chunks-context', name),
    mode: 'production',
    devtool: false,
    entry: { main: './main', outer: './outer', inner: './inner' },
    output: {
      path: path.join(import.meta.dirname, 'dist', name),
      filename: '[name].[contenthash].js',
    },
    optimization: {
      minimize: false,
      concatenateModules: false,
      splitChunks: {
        chunks: 'all',
        minSize: 0,
        cacheGroups: {
          default: false,
          defaultVendors: false,
          shared: { test: /shared-[a-d]\.js$/, minChunks: 2 },
        },
      },
    },
    plugins: [new experiments.VirtualModulesPlugin(modules)],
  })),
);
