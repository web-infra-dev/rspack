'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node14',
  entry: './index.js',
  experiments: { outputModule: true },
  output: { module: true, filename: 'bundle.mjs' },
};
