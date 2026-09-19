import { defineConfig } from '@rspack/cli';

import path from 'node:path';
import { rspack } from '@rspack/core';

export default defineConfig({
  resolve: {
    alias: {
      app: [
        path.join(import.meta.dirname, 'src/main'),
        path.join(import.meta.dirname, 'src/foo'),
      ],
    },
  },
  plugins: [new rspack.ContextReplacementPlugin(/main/, '../override')],
});
