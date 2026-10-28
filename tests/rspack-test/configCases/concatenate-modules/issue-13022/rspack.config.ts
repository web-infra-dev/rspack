import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    entry: {
      index: path.resolve(import.meta.dirname, './index.js'),
    },
    output: {
      library: {
        type: 'var',
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
        type: 'var',
        name: '[name]_doc',
        export: 'default',
      },
    },
    optimization: {
      concatenateModules: true,
    },
  },
]);
