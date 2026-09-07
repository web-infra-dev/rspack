'use strict';

const path = require('path');
const { rspack } = require('@rspack/core');

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

const base = {
  experiments: {
    css: true,
  },
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  module: {
    parser: {
      javascript: {
        importExportsPresence: 'warn',
      },
    },
    rules: baseRules,
  },
  output: {
    uniqueName: 'my-app',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};

/** @type {(env: Env, options: TestOptions) => import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
  {
    ...base,
    name: 'web-development',
    target: 'web',
    mode: 'development',
  },
  {
    ...base,
    name: 'web-production',
    target: 'web',
    mode: 'production',
    plugins: [
      new rspack.ids.DeterministicModuleIdsPlugin({
        maxLength: 3,
        failOnConflict: true,
        fixedLength: true,
        test: (m) => m.type.startsWith('css'),
      }),
      new rspack.experiments.ids.SyncModuleIdsPlugin({
        test: (m) => m.type.startsWith('css'),
        path: path.resolve(testPath, 'module-ids.json'),
        mode: 'create',
      }),
    ],
  },
  {
    ...base,
    dependencies: ['web-development'],
    name: 'node-development',
    target: 'node',
    mode: 'development',
  },
  {
    ...base,
    dependencies: ['web-production'],
    name: 'node-production',
    target: 'node',
    mode: 'production',
    plugins: [
      new rspack.ids.DeterministicModuleIdsPlugin({
        maxLength: 3,
        failOnConflict: true,
        fixedLength: true,
        test: (m) => m.type.startsWith('css'),
      }),
      new rspack.experiments.ids.SyncModuleIdsPlugin({
        test: (m) => m.type.startsWith('css'),
        path: path.resolve(testPath, 'module-ids.json'),
        mode: 'read',
      }),
    ],
  },
  {
    ...base,
    entry: './index-global.js',
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
  },
  {
    ...base,
    entry: './index-global.js',
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
  },
  {
    ...base,
    entry: './index-options.js',
    name: 'web-development-options',
    target: 'web',
    mode: 'development',
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
  },
];
