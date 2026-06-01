'use strict';

const path = require('path');
const { rspack } = require('@rspack/core');

/** @type {(env: Env, options: TestOptions) => import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
  {
    target: 'web',
    mode: 'development',

    module: {
      generator: {
        'css/auto': {
          localIdentName: '[path][name][ext]-[local]',
        },
      },
      rules: [
        {
          test: /\.module\.css$/i,
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
      ],
    },
    node: {
      __dirname: false,
      __filename: false,
    },
  },
  {
    target: 'web',
    mode: 'production',
    output: {
      uniqueName: 'my-app',
    },
    module: {
      generator: {
        'css/auto': {
          localIdentName: '[path][name][ext]-[local]',
        },
      },
      rules: [
        {
          test: /\.module\.css$/i,
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
      ],
    },
    node: {
      __dirname: false,
      __filename: false,
    },
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
];
