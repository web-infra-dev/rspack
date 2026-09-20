import { defineConfig } from '@rspack/cli';

import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  externals: {
    './remoteEntry.js': 'commonjs ./remoteEntry.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'remoteEntry.js',
      library: { type: 'commonjs-module' },
      remotes: {
        remote:
          "promise Promise.resolve().then(() => ({ get: () => Promise.resolve(() => 'remote'), init: () => {} }))",
      },
      exposes: {
        './exposed': './exposed',
      },
      experiments: {
        asyncStartup: true,
      },
    }),
  ],
});
