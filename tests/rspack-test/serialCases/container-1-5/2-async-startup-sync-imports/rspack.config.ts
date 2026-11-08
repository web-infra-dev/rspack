import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

const common = defineConfig({
  entry: {
    main: './index.js',
  },
});

const commonMF = {
  runtime: false as const,
  exposes: {
    './ComponentB': './ComponentB',
    './ComponentC': './ComponentC',
  },
  shared: ['react'],
};

export default defineConfig([
  {
    ...common,
    target: 'async-node',
    output: {
      filename: '[name].js',
      uniqueName: '2-async-startup-sync-imports',
      chunkLoading: 'async-node',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'container',
        library: { type: 'commonjs-module' },
        filename: 'container.js',
        remotes: {
          containerA: '../0-container-full/container.js',
          containerB: './container.js',
        },
        ...commonMF,
        experiments: {
          asyncStartup: true,
        },
      }),
    ],
  },
  {
    ...common,
    output: {
      module: true,
      filename: 'module/[name].mjs',
      uniqueName: '2-async-startup-sync-imports-mjs',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'container',
        library: { type: 'module' },
        filename: 'module/container.mjs',
        remotes: {
          containerA: '../../0-container-full/module/container.mjs',
          containerB: './container.mjs',
        },
        ...commonMF,
        experiments: {
          asyncStartup: true,
        },
      }),
    ],
    target: 'node14',
  },
]);
