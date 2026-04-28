'use strict';

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    target: 'web',
    mode: 'development',
    output: {
      uniqueName: 'my-app',
    },
  },
  {
    target: 'web',
    mode: 'production',
    performance: false,
  },
];
