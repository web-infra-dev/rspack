import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  module: {
    rules: [
      {
        test: path.resolve(import.meta.dirname, 'node_modules/pmodule'),
        sideEffects: true,
      },
      {
        test: path.resolve(import.meta.dirname, 'node_modules/nmodule'),
        sideEffects: false,
      },
    ],
  },
};
