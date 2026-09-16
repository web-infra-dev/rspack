import { container } from '@rspack/core';
import { fileURLToPath } from 'node:url';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

const common = {
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

/** @type {ConstructorParameters<typeof ModuleFederationPlugin>[0]} */
const commonMF = {
  runtime: false,
  exposes: {
    './ComponentB': './ComponentB',
    './ComponentC': './ComponentC',
  },
  shared: ['react'],
};

/** @type {import("@rspack/core").Configuration[]} */
export default [
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
];
