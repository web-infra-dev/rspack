import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  resolve: {
    alias: {
      './wrong2': './ok2',
    },
  },
  module: {
    rules: [
      {
        test: fileURLToPath(import.meta.resolve('./a.js')),
        resolve: {
          alias: {
            './wrong': './ok',
          },
          extensions: ['.js', '.ok.js'],
        },
      },
      {
        test: fileURLToPath(import.meta.resolve('./b.js')),
        resolve: {
          alias: {
            './wrong': './ok',
          },
          extensions: ['...', '.ok.js'],
        },
      },
      {
        test: fileURLToPath(import.meta.resolve('./b.js')),
        resolve: {
          extensions: ['.yes.js', '...'],
        },
      },
    ],
  },
});
