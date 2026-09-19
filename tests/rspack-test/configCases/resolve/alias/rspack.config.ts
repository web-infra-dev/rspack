import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  entry: './index.js',
  resolve: {
    alias: {
      '@b': path.resolve(import.meta.dirname, './a'),
      xx: path.resolve(import.meta.dirname, './a'),
      ignored: path.resolve(import.meta.dirname, './a'),
    },
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        resolve: {
          alias: {
            ignored: false,
          },
        },
      },
      {
        test: /no-alias/,
        resolve: {
          alias: false,
        },
      },
    ],
  },
});
