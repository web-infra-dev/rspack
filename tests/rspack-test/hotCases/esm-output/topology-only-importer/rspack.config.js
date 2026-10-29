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
      cacheGroups: {
        default: false,
        defaultVendors: false,
        dep: {
          test: /[\\/]dep\.js$/,
          name: 'shared',
          chunks: 'all',
          enforce: true,
          minSize: 0,
        },
        lateDep: {
          test: /late-dep\.js$/,
          name: 'late-shared',
          chunks: 'all',
          enforce: true,
          minSize: 0,
        },
      },
    },
  },
};
