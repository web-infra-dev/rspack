'use strict';

const EnvironmentPlugin = require('@rspack/core').EnvironmentPlugin;
const DefinePlugin = require('@rspack/core').DefinePlugin;

process.env.WEBPACK_API_URL = 'https://api.example.com';

const importMetaEnv = {
  NODE_ENV: 'production',
  ENV_VAR_FROM_ENV: 'from_environment_plugin',
  WEBPACK_DOTENV_VAR: 'from_dotenv',
  CUSTOM_VAR: 'custom_value',
};

/** @type {import('@rspack/core').Configuration[]} */
module.exports = {
  // Test 1: NODE_ENV from mode (WebpackOptionsApply)
  mode: 'production',
  // Test 3: DotenvPlugin from .env.test file
  dotenv: {
    template: ['.env.test'],
  },
  plugins: [
    // Test 2: EnvironmentPlugin
    new EnvironmentPlugin({
      ENV_VAR_FROM_ENV: 'from_environment_plugin',
    }),
    // Test 4: DefinePlugin
    new DefinePlugin({
      'import.meta.env': JSON.stringify(importMetaEnv),
      'import.meta.env.CUSTOM_VAR': JSON.stringify('custom_value'),
    }),
  ],
};
