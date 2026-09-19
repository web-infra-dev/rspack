import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        resourceQuery: /raw/,
        type: 'asset/source',
      },
    ],
  },
  resolve: {
    alias: {
      './answer': path.resolve(import.meta.dirname, './answer.js?raw'),
      './no-query-answer': path.resolve(import.meta.dirname, './answer.js'),
    },
  },
});
