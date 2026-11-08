import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPluginV1: ModuleFederationPlugin } = container;

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  mode: 'development',
  optimization: {
    runtimeChunk: 'single',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'A',
      filename: 'container-a.js',
      library: {
        type: 'commonjs-module',
      },
      exposes: {
        '.': './a',
      },
      remoteType: 'commonjs-module',
      remotes: {
        A: './container-a.js',
      },
    }),
  ],
});
