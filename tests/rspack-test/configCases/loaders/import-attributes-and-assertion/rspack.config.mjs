import { fileURLToPath } from 'node:url';
// Rspack don't support assert since it's deprecated

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        with: { type: 'json' },
        loader: fileURLToPath(import.meta.resolve('./loader-with.js')),
      },
    ],
  },
};
