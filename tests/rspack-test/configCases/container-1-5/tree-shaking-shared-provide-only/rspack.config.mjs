import { resolve } from 'node:path';
import { container } from '@rspack/core';
import { fileURLToPath } from 'node:url';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  output: {
    publicPath: 'PUBLIC_PATH',
    chunkFilename: '[id].js',
  },
  resolve: {
    alias: {
      'local-provided': resolve(import.meta.dirname, 'local-provided/index.js'),
    },
  },
  target: 'async-node',
  plugins: [
    new ModuleFederationPlugin({
      name: 'tree_shaking_shared_provide_only',
      filename: 'remoteEntry.js',
      library: {
        type: 'commonjs-module',
        name: 'tree_shaking_shared_provide_only',
      },
      runtimePlugins: [
        fileURLToPath(import.meta.resolve('./runtime-plugin.js')),
      ],
      shared: {
        'provided-only': {
          import: './node_modules/provided-only/index.js',
          requiredVersion: '*',
          version: '1.0.0',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'local-provided': {
          import: './local-provided/index.js',
          requiredVersion: '*',
          version: '2.3.4',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
      },
    }),
  ],
};
