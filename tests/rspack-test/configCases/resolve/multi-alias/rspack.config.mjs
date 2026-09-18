import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    alias: {
      _: [
        path.resolve(import.meta.dirname, 'a'),
        path.resolve(import.meta.dirname, 'b'),
      ],
    },
  },
};
