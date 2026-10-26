import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  output: {
    filename: 'main.mjs',
    library: { type: 'module' },
    module: true,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'consume-shared-output-module',
      shared: {
        'shared-lib': {
          eager: true,
          import: './shared.js',
          requiredVersion: false,
          singleton: true,
        },
      },
    }),
  ],
});
