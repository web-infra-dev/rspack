'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: { main: './index.js' },
  output: {
    module: true,
    library: {
      type: 'module',
    },
    filename: '[name].mjs',
    chunkFormat: 'module',
  },
  externals: {
    'fs-promises': ['module fs', 'promises'],
    'path-posix': ['module path', 'posix'],
  },
  externalsType: 'module',
  optimization: {
    concatenateModules: false,
    usedExports: true,
  },
};
