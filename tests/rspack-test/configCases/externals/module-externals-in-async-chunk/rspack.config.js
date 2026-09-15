'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    external: 'fs',
    external2: 'node:fs',
    external3: 'fs',
  },
  externalsType: 'module-import',
  experiments: {
    outputModule: true,
  },
  output: {
    module: true,
    chunkFormat: 'module',
    chunkFilename: '[name].mjs',
  },
  optimization: {
    moduleIds: 'named',
    concatenateModules: false,
  },
};
