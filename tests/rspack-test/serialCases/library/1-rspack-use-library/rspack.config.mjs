import { rspack } from '@rspack/core';
import path from 'node:path';

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
export default (env, { testPath }) => [
  {
    entry: './default-test.js',
    resolve: {
      alias: {
        library: path.resolve(
          testPath,
          '../0-rspack-create-library/modern-module-non-entry-module-export/main.js',
        ),
      },
    },
    plugins: [
      new rspack.DefinePlugin({
        NAME: JSON.stringify('modern-module export from non-entry module'),
      }),
    ],
  },
];
