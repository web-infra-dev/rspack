'use strict';

const { DefinePlugin } = require('@rspack/core');

/**
 * @param {string | unknown[]} value
 * @returns {import("@rspack/core").Configuration}
 */
const createConfig = (value) => ({
  experiments: {
    env: true,
  },
  optimization: {
    nodeEnv: false,
  },
  plugins: [
    new DefinePlugin({
      'import.meta.env': value,
    }),
  ],
});

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  createConfig(JSON.stringify('production')),
  createConfig('loadEnv()'),
  createConfig([]),
];
