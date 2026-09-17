import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  resolve: {
    alias: {
      '@foo/index': path.resolve(import.meta.dirname, './b'),
      '@foo': path.resolve(import.meta.dirname, './a'), // should not be used
    },
  },
};
