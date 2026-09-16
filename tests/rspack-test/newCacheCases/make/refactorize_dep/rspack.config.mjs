import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
