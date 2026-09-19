import { defineConfig } from '@rspack/cli';

import { container } from '@rspack/core';

const { ModuleFederationPluginV1: ModuleFederationPlugin } = container;

export default defineConfig({
  externals: {
    './container.js': 'commonjs ./container.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      exposes: ['./module'],
    }),
  ],
});
