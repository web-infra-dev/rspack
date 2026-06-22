'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    path: 'node-commonjs path',
  },
  output: {
    module: true,
  },
  devtool: 'eval-source-map',
  target: 'node',
};
