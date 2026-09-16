import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
