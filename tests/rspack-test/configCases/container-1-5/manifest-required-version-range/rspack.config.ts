import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      manifest: true,
      exposes: {
        './module': './module.js',
      },
      shared: {
        // Trim outer whitespace in the inferred range, preserving spaces around ||.
        xreact: {},
        // Configured, overriding the "^2.0.0" in package.json
        yreact: { requiredVersion: '^2.0.0 || ^3.0.0' },
      },
    }),
  ],
});
