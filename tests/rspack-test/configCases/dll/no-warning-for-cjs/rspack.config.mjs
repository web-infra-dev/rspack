import { rspack } from '@rspack/core';
import path from 'node:path';

const dllManifest = path.resolve(
  import.meta.dirname,
  '../../../js/config/dll/no-warning-for-cjs/manifest.json',
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
      new rspack.DllReferencePlugin({
        manifest: dllManifest,
        sourceType: 'commonjs2',
        scope: 'dll',
        name: './lib-dll.js',
      }),
    ],
  },
];
