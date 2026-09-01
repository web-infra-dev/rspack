'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    environment: {
      logicalAssignment: false,
    },
  },
  externals: {
    module: 'commonjs module',
    fs: 'commonjs fs',
  },
};
