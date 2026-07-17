'use strict';

const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    env: true,
  },
  plugins: [
    new DefinePlugin({
      'typeof import.meta.env': JSON.stringify('custom'),
    }),
  ],
};
