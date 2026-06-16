'use strict';

const EnvironmentPlugin = require('@rspack/core').EnvironmentPlugin;
const DefinePlugin = require('@rspack/core').DefinePlugin;

/** @type {import('@rspack/core').Configuration[]} */
module.exports = {
  // Test 1: NODE_ENV from mode (WebpackOptionsApply)
  mode: 'production',
  plugins: [
    // Test 2: EnvironmentPlugin
    new EnvironmentPlugin({
      ENV_VAR_FROM_ENV: 'from_environment_plugin',
    }),
    // Test 3: DefinePlugin
    new DefinePlugin({
      'import.meta.env.CUSTOM_VAR': JSON.stringify('custom_value'),
    }),
  ],
};
