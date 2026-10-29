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
};
