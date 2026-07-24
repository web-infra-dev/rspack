'use strict';

const { EnvironmentPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  module: {
    rules: [
      {
        test: /disabled\.js$/,
        parser: {
          importMeta: {
            env: false,
          },
        },
      },
    ],
  },
  plugins: [
    new EnvironmentPlugin({
      AAA: 'aaa',
    }),
  ],
  experiments: {
    env: true,
    outputModule: true,
  },
  output: {
    module: true,
    chunkFormat: 'module',
  },
  externals: {
    fs: 'commonjs fs',
  },
};
