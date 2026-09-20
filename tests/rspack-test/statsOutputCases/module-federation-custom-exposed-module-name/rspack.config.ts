import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  mode: 'production',
  entry: './index.js',
  output: {
    filename: '[name]_bundle.js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      exposes: {
        './entry': {
          import: './entry',
          name: 'custom-entry',
        },
      },
    }),
  ],
});
