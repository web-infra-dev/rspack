'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    module: true,
    enabledLibraryTypes: ['module'],
    chunkFormat: 'module',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.mjs',
    publicPath: 'auto',
    environment: {
      nodePrefixForCoreModules: false,
    },
  },
  optimization: {
    splitChunks: {
      minSize: 0,
      cacheGroups: {
        dep: { test: /dep/, name: 'dep', chunks: 'all', enforce: true },
      },
    },
  },
};
