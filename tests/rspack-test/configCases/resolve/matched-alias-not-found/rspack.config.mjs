import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    alias: {
      m1: path.resolve(import.meta.dirname, 'node_modules', 'm2', 'mod.js'),
    },
  },
};
