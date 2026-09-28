import path from 'node:path';

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    entry: {
      index: path.resolve(import.meta.dirname, './index.js'),
    },
    output: {
      library: {
        name: '[name]',
        export: 'default',
      },
    },
    optimization: {
      concatenateModules: true,
    },
  },
  {
    entry: {
      index: path.resolve(import.meta.dirname, './index.js'),
    },
    output: {
      library: {
        name: '[name]_doc',
        export: 'default',
      },
    },
    optimization: {
      concatenateModules: true,
    },
  },
];
