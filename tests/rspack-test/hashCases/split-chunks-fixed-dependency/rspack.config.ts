import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { experiments } from '@rspack/core';

const root = path.resolve('/rspack-context');
const fixedDependency = path.join(root, 'm', 'vendor', 'shared-v.js');

export default defineConfig(
  ['a', 'z'].map((name) => ({
    name,
    context: path.join(root, name, 'app'),
    mode: 'production',
    devtool: false,
    entry: { main: './main', local: './shared-l', vendor: 'fixed-dep' },
    resolve: { alias: { 'fixed-dep': fixedDependency } },
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
          shared: { test: /shared-[lv]\.js$/, minChunks: 2 },
        },
      },
    },
    plugins: [
      new experiments.VirtualModulesPlugin({
        'main.js': "import(/* webpackChunkName: 'lazy' */ './lazy.js');",
        'lazy.js': "import './shared-l'; import 'fixed-dep';",
        'shared-l.js': "globalThis.collect('l');",
        [fixedDependency]: "globalThis.collect('v');",
      }),
    ],
  })),
);
