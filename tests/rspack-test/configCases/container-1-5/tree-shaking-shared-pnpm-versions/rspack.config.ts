import { defineConfig } from '@rspack/cli';

import { container } from '@rspack/core';
import { fileURLToPath } from 'node:url';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

export default defineConfig({
  entry: './index.js',
  output: {
    publicPath: 'PUBLIC_PATH',
    chunkFilename: '[id].js',
  },
  target: 'async-node',
  plugins: [
    new ModuleFederationPlugin({
      name: 'tree_shaking_shared_pnpm_versions',
      filename: 'remoteEntry.js',
      library: {
        type: 'commonjs-module',
        name: 'tree_shaking_shared_pnpm_versions',
      },
      runtimePlugins: [
        fileURLToPath(import.meta.resolve('./runtime-plugin.js')),
      ],
      shared: {
        'ui-lib': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
      },
    }),
  ],
});
