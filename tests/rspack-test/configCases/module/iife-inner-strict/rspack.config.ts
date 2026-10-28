import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    entry: ['./index.mjs'],
    output: {
      module: false,
    },
  },
  {
    name: 'test-output',
    entry: './test.js',
    output: {
      filename: 'test.js',
    },
  },
]);
