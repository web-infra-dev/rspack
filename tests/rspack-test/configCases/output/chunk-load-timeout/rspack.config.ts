import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    entry: './a',
    target: 'web',
    output: {
      filename: 'a.js',
      chunkLoadTimeout: 1234000,
    },
  },
  {
    entry: './index',
  },
]);
