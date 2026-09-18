import { rspack } from '@rspack/core';
import path from 'node:path';
import { readFileSync } from 'node:fs';

const dllManifest = path.resolve(
  import.meta.dirname,
  '../../../js/config/dll/numeric-module-id/manifest.json',
);

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    name: 'create-dll',
    entry: './lib.js',
    output: {
      filename: 'lib-dll.js',
      library: {
        type: 'commonjs2',
      },
    },
    optimization: {
      moduleIds: 'deterministic',
      chunkIds: 'deterministic',
    },
    plugins: [
      new rspack.DllPlugin({
        path: dllManifest,
        entryOnly: false,
      }),
    ],
  },
  {
    name: 'use-dll',
    dependencies: ['create-dll'],
    entry: './main.js',
    plugins: [
      function (compiler) {
        compiler.hooks.beforeRun.tap('test', () => {
          new rspack.DllReferencePlugin({
            manifest: JSON.parse(readFileSync(dllManifest, 'utf-8')),
            sourceType: 'commonjs2',
            scope: 'dll',
            name: './lib-dll.js',
          }).apply(compiler);
        });
      },
    ],
  },
];
