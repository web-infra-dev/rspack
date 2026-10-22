import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';
import { fileURLToPath } from 'node:url';

// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

const commonConfig = defineConfig({
  optimization: {
    minimize: false,
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    ignoreBrowserWarnings: true,
    publicPath: '/',
    chunkFilename: '[id].js',
  },
  target: 'async-node',
});

export default defineConfig([
  {
    ...commonConfig,
    entry: './index.js',
    plugins: [
      new ModuleFederationPlugin({
        name: 'remote_array_share_scope_host',
        remotes: {
          'remote-alias': {
            external:
              'remote_array_share_scope_provider@http://localhost:3001/remoteEntry.js',
            shareScope: ['scope1', 'scope3'],
          },
        },
        runtimePlugins: [
          fileURLToPath(import.meta.resolve('./runtime-plugin.js')),
        ],
        shared: {
          '@scope-sc/ui-lib': {
            requiredVersion: '*',
            shareScope: 'scope1',
          },
          '@scope-sc/ui-lib2': {
            requiredVersion: '*',
            shareScope: 'scope3',
          },
          '@scope-sc/ui-lib3': {
            requiredVersion: '*',
          },
        },
      }),
    ],
  },
  {
    ...commonConfig,
    entry: {
      output: './Expose.js',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'remote_array_share_scope_provider',
        manifest: true,
        filename: 'remoteEntry.js',
        shareScope: ['scope1', 'scope2', 'scope3'],
        library: {
          type: 'commonjs-module',
          name: 'remote_array_share_scope_provider',
        },
        exposes: {
          './Expose': './Expose.js',
        },
        runtimePlugins: [
          fileURLToPath(import.meta.resolve('./runtime-plugin.js')),
        ],
        shared: {
          '@scope-sc/ui-lib': {
            requiredVersion: '*',
            shareScope: 'scope1',
          },
          '@scope-sc/ui-lib2': {
            requiredVersion: '*',
            shareScope: 'scope3',
          },
          '@scope-sc/ui-lib3': {
            requiredVersion: '*',
          },
        },
      }),
    ],
  },
]);
