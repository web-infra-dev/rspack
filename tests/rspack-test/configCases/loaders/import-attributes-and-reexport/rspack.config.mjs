import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        with: { type: 'RANDOM' },
        use: fileURLToPath(import.meta.resolve('./test-loader.mjs')),
      },
    ],
  },
};
