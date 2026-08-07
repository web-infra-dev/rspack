'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  devtool: false,
  optimization: {
    concatenateModules: true,
    minimize: false,
    moduleIds: 'named',
    chunkIds: 'named',
  },
  stats: {
    optimizationBailout: true,
  },
};
