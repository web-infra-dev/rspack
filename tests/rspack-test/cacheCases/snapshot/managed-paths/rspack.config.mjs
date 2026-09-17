import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      managedPaths: [path.join(import.meta.dirname, './test_lib')],
    },
  },
};
