'use strict';

const path = require('path');
const { rspack } = require('@rspack/core');

const externals = {
  fs: 'node-commonjs fs',
  path: 'node-commonjs path',
};

const ignoreCssParseWarnings = [
  /CSS parse warning: (?!Broken '@value' at-rule)/,
];

/** @type {NonNullable<import('@rspack/core').Configuration['module']>['rules']} */
const baseRules = [
  {
    test: /\.css$/i,
    type: 'css/auto',
  },
  {
    test: /\.my-css$/i,
    type: 'css/auto',
  },
  {
    test: /\.invalid$/i,
    type: 'css/auto',
  },
];

/** @type {import('@rspack/core').Configuration} */
const base = {
  externals,
  ignoreWarnings: ignoreCssParseWarnings,
  module: {
    parser: {
      javascript: {
        importExportsPresence: 'warn',
      },
    },
    rules: baseRules,
  },
};

const deterministicCssPlugins = (testPath, mode) => [
  new rspack.ids.DeterministicModuleIdsPlugin({
    maxLength: 3,
    failOnConflict: true,
    fixedLength: true,
  }),
  new rspack.experiments.ids.SyncModuleIdsPlugin({
    path: path.resolve(testPath, 'module-ids.json'),
    mode,
  }),
];

/** @type {(env: Env, options: TestOptions) => import('@rspack/core').Configuration[]} */
module.exports = (env, { testPath }) => [
  {
    ...base,
    name: 'web-development',
    target: 'web',
    mode: 'development',
    output: {
      uniqueName: 'my-app',
    },
    node: {
      __dirname: false,
      __filename: false,
    },
  },
  {
    ...base,
    name: 'web-production',
    target: 'web',
    mode: 'production',
    output: {
      uniqueName: 'my-app',
    },
    incremental: {
      moduleIds: false,
    },
    optimization: {
      concatenateModules: false,
      moduleIds: false,
    },
    node: {
      __dirname: false,
      __filename: false,
    },
    plugins: deterministicCssPlugins(testPath, 'create'),
  },
  {
    ...base,
    dependencies: ['web-development'],
    name: 'node-development',
    target: 'node',
    mode: 'development',
    output: {
      uniqueName: 'my-app',
    },
  },
  {
    ...base,
    dependencies: ['web-production'],
    name: 'node-production',
    target: 'node',
    mode: 'production',
    output: {
      uniqueName: 'my-app',
    },
    incremental: {
      moduleIds: false,
    },
    optimization: {
      concatenateModules: false,
      moduleIds: false,
    },
    plugins: deterministicCssPlugins(testPath, 'read'),
  },
  {
    entry: './index-global.js',
    externals,
    ignoreWarnings: ignoreCssParseWarnings,
    name: 'web-development-global',
    target: 'web',
    mode: 'development',
    module: {
      rules: [
        {
          test: /\.css$/i,
          type: 'css/global',
        },
        {
          test: /\.my-css$/i,
          type: 'css/global',
        },
        {
          test: /\.invalid$/i,
          type: 'css/global',
        },
      ],
    },
    output: {
      uniqueName: 'my-app',
    },
    node: {
      __dirname: false,
      __filename: false,
    },
  },
  {
    entry: './index-global.js',
    externals,
    ignoreWarnings: ignoreCssParseWarnings,
    name: 'web-production-global',
    target: 'web',
    mode: 'production',
    module: {
      rules: [
        {
          test: /\.css$/i,
          type: 'css/global',
        },
        {
          test: /\.my-css$/i,
          type: 'css/global',
        },
        {
          test: /\.invalid$/i,
          type: 'css/global',
        },
      ],
    },
    output: {
      uniqueName: 'my-app',
    },
    node: {
      __dirname: false,
      __filename: false,
    },
  },
  {
    ...base,
    entry: './index-options.js',
    name: 'web-development-options',
    target: 'web',
    mode: 'development',
    output: {
      uniqueName: 'my-app',
    },
    module: {
      rules: [
        ...baseRules,
        {
          test: /style\.module\.css$/,
          type: 'css/auto',
          parser: {
            animation: false,
            customIdents: false,
            dashedIdents: false,
            container: false,
            function: false,
            grid: false,
          },
        },
      ],
    },
    node: {
      __dirname: false,
      __filename: false,
    },
  },
];
