import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
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
});
