import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default [
  {
    entry: {
      index: { import: './index.js', filename: 'bundle0.js' },
    },
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
    output: {
      library: {
        type: 'commonjs',
      },
    },
    plugins: [
      new RslibPlugin({
        interceptApiPlugin: true,
      }),
    ],
  },
  {
    entry: {
      index: { import: './module.js', filename: 'bundle1.mjs' },
    },
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
    externals: {
      'node:module': 'module-import node:module',
    },
    output: {
      module: true,
      library: {
        type: 'modern-module',
      },
    },
    plugins: [
      new RslibPlugin({
        interceptApiPlugin: true,
      }),
    ],
  },
  {
    entry: './test.js',
    externals: {
      './bundle0.js': 'commonjs ./bundle0.js',
    },
    target: 'node',
    node: {
      __filename: false,
      __dirname: false,
    },
  },
];
