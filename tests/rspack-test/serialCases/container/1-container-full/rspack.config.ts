import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPluginV1: ModuleFederationPlugin } = container;

const common = defineConfig({
  entry: {
    main: './index.js',
  },
  optimization: {
    runtimeChunk: 'single',
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
    output: {
      filename: '[name].js',
      uniqueName: '1-container-full',
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
      }),
    ],
  },
  {
    ...common,
    output: {
      module: true,
      filename: 'module/[name].mjs',
      uniqueName: '1-container-full-mjs',
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
      }),
    ],
    target: 'node14',
  },
]);
