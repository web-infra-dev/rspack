import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    alias: {
      app: [
        path.join(import.meta.dirname, 'src/main'),
        path.join(import.meta.dirname, 'src/foo'),
      ],
    },
  },
};
