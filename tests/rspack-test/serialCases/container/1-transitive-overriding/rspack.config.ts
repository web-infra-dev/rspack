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
      name: 'container-no-shared',
      library: { type: 'commonjs-module' },
      filename: 'container-no-shared.js',
      exposes: ['./a', './b', './modules', './modules-from-remote'],
      remotes: {
        'container-with-shared':
          '../0-transitive-overriding/container-with-shared.js',
        'container-no-shared': './container-no-shared.js',
      },
    }),
  ],
});
