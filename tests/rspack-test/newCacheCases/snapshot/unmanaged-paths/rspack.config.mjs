import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      managedPaths: [path.join(import.meta.dirname, './packages')],
      unmanagedPaths: [
        path.join(import.meta.dirname, 'packages/test_lib/unmanaged'),
      ],
    },
  },
};
