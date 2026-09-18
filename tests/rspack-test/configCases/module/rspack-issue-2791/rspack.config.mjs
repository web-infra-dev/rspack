import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
