import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    modules: [
      path.resolve(import.meta.dirname, 'a'),
      path.resolve(import.meta.dirname, 'b'),
    ],
    alias: {
      [path.resolve(import.meta.dirname, 'a', 'foo')]: false,
    },
  },
};
