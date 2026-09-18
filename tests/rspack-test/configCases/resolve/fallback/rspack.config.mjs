import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    alias: {
      '#': path.resolve(import.meta.dirname, '#'),
    },
    fallback: {
      './b': path.resolve(import.meta.dirname, 'a'),
    },
  },
};
