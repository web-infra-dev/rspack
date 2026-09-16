import { container } from '@rspack/core';
import { fileURLToPath } from 'node:url';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */

export default {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  entry: './index.js',
  output: {
    ignoreBrowserWarnings: true,
  },
  optimization: {
    minimize: true,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    publicPath: '/',
    chunkFilename: '[id].js',
  },
  target: 'async-node',
  plugins: [
    new ModuleFederationPlugin({
      name: 'tree_shaking_share',
      manifest: true,
      filename: 'remoteEntry.js',
      library: {
        type: 'commonjs-module',
        name: 'tree_shaking_share',
      },
      exposes: {
        './App': './App.js',
      },
      runtimePlugins: [
        fileURLToPath(import.meta.resolve('./runtime-plugin.js')),
      ],
      shared: {
        '@scope-sc/ui-lib': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-es': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-dynamic-specific-export': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-dynamic-default-export': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
        'ui-lib-side-effect': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
      },
    }),
  ],
};
