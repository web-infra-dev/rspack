'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = [
  {
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    node: false,
    mode: 'production',
    devtool: false,
    optimization: {
      concatenateModules: true,
    },
  },
  {
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    node: false,
    mode: 'production',
    devtool: false,
    optimization: {
      concatenateModules: false,
    },
  },
  {
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    node: false,
    mode: 'production',
    devtool: 'eval',
    optimization: {
      concatenateModules: true,
    },
  },
  {
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    node: false,
    mode: 'production',
    devtool: 'eval',
    optimization: {
      concatenateModules: false,
    },
  },
];
