import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    modules: [
      'node_modules',
      path.resolve(import.meta.dirname, './node_modules'),
    ],
  },
};
