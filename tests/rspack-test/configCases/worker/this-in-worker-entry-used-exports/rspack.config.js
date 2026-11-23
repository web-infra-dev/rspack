'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    minimize: false,
    usedExports: true,
    mangleExports: true,
  },
};
