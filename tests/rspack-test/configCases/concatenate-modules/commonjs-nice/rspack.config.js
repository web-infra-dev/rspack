'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  devtool: false,
  optimization: {
    concatenateModules: { commonjs: true },
    minimize: false,
    usedExports: true,
    moduleIds: 'named',
    chunkIds: 'named',
  },
};
