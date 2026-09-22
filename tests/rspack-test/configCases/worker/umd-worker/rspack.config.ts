import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig((_env, { testPath }) => [
  {
    name: 'library',
    entry: './library.js',
    target: 'web',
    output: {
      library: {
        name: 'library',
        type: 'umd',
      },
    },
  },
  {
    name: 'build',
    dependencies: ['library'],
    entry: './index.js',
    target: 'web',
    resolve: {
      alias: {
        library: path.resolve(testPath, './bundle0.js'),
      },
    },
  },
]);
