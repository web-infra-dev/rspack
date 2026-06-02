'use strict';

const path = require('path');
const { rspack } = require('@rspack/core');

/** @type {(env: Env, options: TestOptions) => import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => {
  const readIdsPath = path.resolve(testPath, 'read-module-ids.json');
  const mergeIdsPath = path.resolve(testPath, 'merge-module-ids.json');
  const updateIdsPath = path.resolve(testPath, 'update-module-ids.json');

  return [
    {
      name: 'create-for-read',
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
          path: readIdsPath,
          mode: 'create',
        }),
      ],
    },
    {
      name: 'read',
      dependencies: ['create-for-read'],
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
          path: readIdsPath,
          mode: 'read',
        }),
      ],
    },
    {
      name: 'create-for-merge',
      entry: './seed.js',
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
          path: mergeIdsPath,
          mode: 'create',
        }),
      ],
    },
    {
      name: 'merge',
      dependencies: ['create-for-merge'],
      entry: './merge.js',
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
          path: mergeIdsPath,
          mode: 'merge',
        }),
      ],
    },
    {
      name: 'create-for-update',
      entry: './seed-update.js',
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
          path: updateIdsPath,
          mode: 'create',
        }),
      ],
    },
    {
      name: 'update',
      dependencies: ['create-for-update'],
      entry: './update.js',
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
          path: updateIdsPath,
          mode: 'update',
        }),
      ],
    },
  ];
};
