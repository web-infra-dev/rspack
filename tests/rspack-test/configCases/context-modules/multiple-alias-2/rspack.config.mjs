import path from 'node:path';
import { rspack } from '@rspack/core';

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
  plugins: [new rspack.ContextReplacementPlugin(/main/, '../override')],
};
