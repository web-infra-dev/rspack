import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  plugins: [
    new ModuleFederationPlugin({
      remoteType: 'commonjs-module',
      remotes: {
        containerB: '../1-container-full/container.js',
      },
      shared: ['@rspack/mocked-react'],
    }),
  ],
});
