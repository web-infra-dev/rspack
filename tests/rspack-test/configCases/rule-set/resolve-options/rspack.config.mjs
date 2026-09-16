import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
