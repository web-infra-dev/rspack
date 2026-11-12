import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

const common = {
  name: 'container',
  exposes: {
    './ComponentA': {
      import: './ComponentA',
    },
  },
  shared: {
    react: {
      version: false as const,
      requiredVersion: false as const,
    },
  },
};

export default defineConfig([
  {
    entry: {
      main: './index.js',
      other: './index-2.js',
    },
    output: {
      filename: '[name].js',
      uniqueName: 'ref-hoist',
    },
    optimization: {
      runtimeChunk: 'single',
      moduleIds: 'named',
    },
    plugins: [
      new ModuleFederationPlugin({
        runtime: false as const,
        library: { type: 'commonjs-module' },
        filename: 'container.js',
        remotes: {
          containerA: {
            external: './container.js',
          },
          containerB: {
            external: '../0-container-full/container.js',
          },
        },
        ...common,
      }),
    ],
  },
  {
    entry: {
      main: './index.js',
      other: './index-2.js',
    },
    optimization: {
      runtimeChunk: 'single',
      moduleIds: 'named',
    },
    output: {
      filename: 'module/[name].mjs',
      uniqueName: 'ref-hoist-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        runtime: false as const,
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          containerA: {
            external: './container.mjs',
          },
          containerB: {
            external: '../../0-container-full/module/container.mjs',
          },
        },
        ...common,
      }),
    ],
    target: 'node14',
  },
]);
