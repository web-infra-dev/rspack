import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    name: 'changing',
    entry: './index.js',
    output: {
      filename: './bundle.js',
    },
  },
  {
    name: 'static',
    entry: './static-file.js',
    output: {
      filename: './static.js',
    },
  },
]);
