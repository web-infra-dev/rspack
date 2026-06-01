'use strict';

const path = require('path');
const { rspack } = require('@rspack/core');

/** @type {(env: Env, options: TestOptions) => import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => {
  const idsPath = path.resolve(testPath, 'module-ids.json');

  return [
    {
      name: 'create',
      mode: 'production',
      target: 'node',
      node: {
        __dirname: false,
      },
      optimization: {
        moduleIds: false,
        concatenateModules: false,
      },
      plugins: [
        new rspack.ids.DeterministicModuleIdsPlugin({
          maxLength: 3,
          fixedLength: true,
          failOnConflict: true,
        }),
        new rspack.experiments.ids.SyncModuleIdsPlugin({
          path: idsPath,
          mode: 'create',
        }),
      ],
    },
    {
      name: 'read',
      dependencies: ['create'],
      mode: 'production',
      target: 'node',
      node: {
        __dirname: false,
      },
      optimization: {
        moduleIds: false,
        concatenateModules: false,
      },
      plugins: [
        new rspack.ids.DeterministicModuleIdsPlugin({
          maxLength: 3,
          fixedLength: true,
          failOnConflict: true,
          salt: 1,
        }),
        new rspack.experiments.ids.SyncModuleIdsPlugin({
          path: idsPath,
          mode: 'read',
        }),
      ],
    },
  ];
};
