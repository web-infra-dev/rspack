import { defineConfig } from '@rspack/cli';
import { type Configuration, container } from '@rspack/core';
import { fileURLToPath } from 'node:url';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

const common: Configuration = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    vm: 'node-commonjs vm',
  },
  entry: {
    main: './index.js',
  },
  target: 'async-node',
  optimization: {
    runtimeChunk: 'single',
  },
};

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
        runtimePlugins: [
          fileURLToPath(import.meta.resolve('./runtimePlugin.js')),
        ],
        filename: 'container.js',
        remotes: {
          containerA: '../0-container-full/container.js',
        },
        ...commonMF,
      }),
    ],
  },
]);
