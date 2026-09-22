import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { container } from '@rspack/core';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { ModuleFederationPlugin } = container;

const implementation = require.resolve('@module-federation/runtime-tools', {
  paths: [
    path.dirname(
      fileURLToPath(import.meta.resolve('@rspack/core/package.json')),
    ),
  ],
});

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared/,
          name: 'shared',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      implementation,
      manifest: true,
      exposes: {
        './expose-a': {
          import: './expose-a.js',
          name: '__federation_expose_expose-a',
        },
        './expose-b': {
          import: './expose-b.js',
          name: '__federation_expose_expose-b',
        },
      },
    }),
  ],
});
