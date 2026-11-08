import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  output: {
    filename: '[name].js',
    uniqueName: 'provide-sharing-extra-data',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container-provide-sharing-extra-data',
      shared: {
        react: {
          version: '0.1.2',
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
        },
      },
    }),
  ],
});
