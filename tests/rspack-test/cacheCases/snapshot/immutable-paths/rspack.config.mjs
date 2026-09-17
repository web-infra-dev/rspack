import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, './file.js')],
    },
  },
};
