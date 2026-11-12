import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const { ModuleFederationPluginV1: ModuleFederationPlugin } = rspack.container;

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
  },
  plugins: [
    new ModuleFederationPlugin({
      shared: {
        table: {
          requiredVersion: '=1.0.0',
        },
        cell: {
          requiredVersion: '=1.0.0',
        },
        row: {
          requiredVersion: '=1.0.0',
        },
        templater: {
          requiredVersion: '=1.0.0',
        },
      },
    }),
  ],
});
