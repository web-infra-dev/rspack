import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    concatenateModules: false,
  },
  output: {
    uniqueName: '2-transitive-overriding',
  },
  plugins: [
    new ModuleFederationPlugin({
      remoteType: 'commonjs-module',
      remotes: {
        'container-no-shared':
          '../1-transitive-overriding/container-no-shared.js',
      },
      shared: {
        './shared': {
          shareKey: 'shared',
          version: '2',
        },
      },
    }),
  ],
});
