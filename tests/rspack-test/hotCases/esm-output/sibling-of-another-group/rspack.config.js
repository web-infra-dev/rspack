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
        c: { test: /[\\/]c\.js$/, name: 'c', chunks: 'async', enforce: true },
        x: { test: /[\\/]x\.js$/, name: 'x', chunks: 'async', enforce: true },
      },
    },
  },
};
