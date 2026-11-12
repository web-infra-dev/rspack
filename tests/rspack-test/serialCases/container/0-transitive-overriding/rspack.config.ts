import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPluginV1: ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container-with-shared',
      library: { type: 'commonjs-module' },
      filename: 'container-with-shared.js',
      exposes: ['./a', './b', './modules'],
      remotes: {
        'container-with-shared': './container-with-shared.js',
      },
      shared: {
        './shared': {
          shareKey: 'shared',
          version: '1',
        },
      },
    }),
  ],
});
