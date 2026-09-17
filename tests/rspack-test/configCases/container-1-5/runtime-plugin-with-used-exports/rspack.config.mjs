import path from 'node:path';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
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
          version: false,
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
          version: '0.1.2',
        },
      },
      runtimePlugins: [
        path.resolve(import.meta.dirname, 'runtime-plugin.js'),
        path.resolve(import.meta.dirname, 'runtime-plugin-esm.js'),
      ],
    }),
  ],
};
