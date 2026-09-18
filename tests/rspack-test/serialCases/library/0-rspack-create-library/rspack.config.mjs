import { rspack } from '@rspack/core';
import path from 'node:path';

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
export default (env, { testPath }) => [
  {
    entry: {
      main: './modern-module-non-entry-module-export/index.js',
    },
    output: {
      module: true,
      chunkFormat: 'module',
      filename: 'modern-module-non-entry-module-export/[name].js',
      library: {
        type: 'modern-module',
      },
    },
    optimization: {
      concatenateModules: true,
      avoidEntryIife: true,
    },
  },
];
