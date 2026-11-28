'use strict';

const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: './index.js',
  optimization: {
    minimize: false,
    usedExports: true,
    concatenateModules: false,
  },
  plugins: [
    new DefinePlugin({
      DEFINED_USED: JSON.stringify('defined-used'),
      DEFINED_UNUSED: JSON.stringify('defined-unused'),
    }),
  ],
};
