import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    providedExports: true,
    usedExports: true,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'runtime-plugin-with-used-exports',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      shared: {
        react: {
          version: '0.1.2',
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
        },
      },
      runtimePlugins: [
        path.resolve(import.meta.dirname, 'runtime-plugin.js'),
        path.resolve(import.meta.dirname, 'runtime-plugin-esm.js'),
      ],
    }),
  ],
});
