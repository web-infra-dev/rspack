'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  output: {
    module: true,
    enabledLibraryTypes: ['module'],
    chunkFormat: 'module',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.mjs',
  },
  optimization: {
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        dep: { test: /dep/, name: 'dep', chunks: 'initial', enforce: true },
      },
    },
  },
};
