import { defineConfig } from '@rspack/cli';

import path from 'node:path';
export default defineConfig({
  module: {
    rules: [
      {
        test: path.resolve(import.meta.dirname, 'index.js'),
        loader: './loader.mjs',
      },
    ],
  },
});
