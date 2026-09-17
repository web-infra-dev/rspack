import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default [
  {
    entry: {
      importAttributes: './importAttributes.mjs',
    },
    output: {
      filename: '[name].js',
      module: true,
    },
    externalsType: 'module',
    externals: ['./a.json'],
    optimization: {
      minimize: true,
      minimizer: [new rspack.SwcJsMinimizerRspackPlugin()],
    },
  },
  {
    entry: {
      main: './index.js',
    },
    output: {
      filename: '[name].js',
    },
    externalsPresets: {
      node: true,
    },
  },
];
