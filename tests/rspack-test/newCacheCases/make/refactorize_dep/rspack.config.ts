import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
  },
  optimization: {
    providedExports: false,
  },
  resolve: {
    alias: {
      data: [
        path.resolve(import.meta.dirname, './data1.js'),
        path.resolve(import.meta.dirname, './data2.js'),
      ],
    },
  },
});
