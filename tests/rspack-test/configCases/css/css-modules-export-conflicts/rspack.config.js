'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  mode: 'development',
  experiments: {
    css: true,
  },
  module: {
    rules: [{ test: /\.module\.css$/i, type: 'css/auto' }],
    generator: {
      'css/auto': {
        // Predictable scoping so JS-export assertions can compare against a fixed string.
        localIdentName: '[local]',
      },
    },
  },
};
