import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  entry: './src/index.js',
  plugins: [
    new ModuleFederationPlugin({
      shared: ['./src/shared.js'],
    }),
  ],
});
