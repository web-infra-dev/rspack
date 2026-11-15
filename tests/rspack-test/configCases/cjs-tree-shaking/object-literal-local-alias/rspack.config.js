'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: './index.js',
  optimization: {
    minimize: false,
    usedExports: true,
    concatenateModules: false,
  },
};
