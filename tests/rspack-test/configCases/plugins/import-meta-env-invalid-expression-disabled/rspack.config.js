'use strict';

const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new DefinePlugin({
      'import.meta.env': 'loadEnv()',
    }),
  ],
};
