'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  entry: { main: './index.js', other: './other.js' },
  output: {
    module: true,
    enabledLibraryTypes: ['module'],
    chunkFormat: 'module',
    filename: '[name].mjs',
    chunkFilename: '[name].chunk.mjs',
  },
};
