'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: { main: './index.js', other: './other.js' },
  output: { filename: '[name].js' },
};
