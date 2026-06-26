'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  mode: 'development',
  experiments: {
    css: true,
  },
  output: {
    uniqueName: 'value-overriding',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.module\.css$/i,
        type: 'css/module',
      },
    ],
  },
};
