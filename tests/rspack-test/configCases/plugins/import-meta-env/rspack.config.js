'use strict';

const { DefinePlugin, EnvironmentPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  // Test 1: NODE_ENV from mode (WebpackOptionsApply)
  mode: 'production',
  experiments: {
    env: true,
  },
  plugins: [
    // Test 2: EnvironmentPlugin
    new EnvironmentPlugin({
      ENV_VAR_FROM_ENV: 'from_environment_plugin',
      FROM_ENVIRONMENT_PLUGIN: 'from-environment-plugin',
    }),
    new DefinePlugin({
      'import.meta.env': JSON.stringify({
        MODE: 'production',
        FEATURE: 'enabled',
        NESTED: {
          FROM_OBJECT: 'nested-object',
        },
        'DOT.KEY': 'dot-key',
      }),
    }),
    // Test 3: DefinePlugin
    new DefinePlugin({
      'import.meta.env.CUSTOM_VAR': JSON.stringify('custom_value'),
      'import.meta.env.ONLY_IMPORT_META': JSON.stringify('only_import_meta'),
      'import.meta.env.ORDERED_VAR': JSON.stringify('first_define_plugin'),
      'import.meta.env.__proto__': JSON.stringify('proto_value'),
      'import.meta.env.PER_KEY': JSON.stringify('per-key'),
      'import.meta.env.NESTED.PER_KEY': JSON.stringify('nested-per-key'),
      'import.meta.env.OBJECT_FORM': {
        VALUE: JSON.stringify('object-form'),
        ARRAY: [JSON.stringify('a'), JSON.stringify('b')],
        'DOT.KEY': JSON.stringify('object-dot'),
      },
      'process.env.PROCESS_ONLY': JSON.stringify('process_only'),
      'import.meta.env.EXPLICIT_UNDEFINED': 'undefined',
      'import.meta.env.DYNAMIC': 'loadEnv()',
      'typeof import.meta.env.TYPEOF_DEFINED': JSON.stringify('string'),
      'import.meta.env.RAW_OBJECT_CODE': JSON.stringify({ x: 1 }),
      'import.meta.env.UNRELATED_SIDE_EFFECT':
        'globalThis.__IMPORT_META_ENV_MISSING_SIDE_EFFECT__ = true',
      'import.meta.env.NESTED_MISSING_WITH_SIDE_EFFECT.UNUSED':
        'globalThis.__IMPORT_META_ENV_MISSING_SIDE_EFFECT__ = true',
      'import.meta.env.DESTRUCTURED_USED': JSON.stringify('destructured-used'),
      'import.meta.env.DESTRUCTURED_UNUSED':
        'globalThis.__IMPORT_META_ENV_UNUSED__ = true',
      'import.meta.env.NESTED_DESTRUCTURING': {
        USED: JSON.stringify('nested-used'),
        UNUSED: 'globalThis.__IMPORT_META_ENV_NESTED_UNUSED__ = true',
      },
    }),
    new DefinePlugin({
      'import.meta.env.ORDERED_VAR': JSON.stringify('second_define_plugin'),
    }),
  ],
};
