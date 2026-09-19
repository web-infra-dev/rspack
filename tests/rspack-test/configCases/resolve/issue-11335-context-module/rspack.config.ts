import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  resolve: {
    alias: {
      app: [
        path.join(import.meta.dirname, 'src/main'),
        path.join(import.meta.dirname, 'src/foo'),
      ],
    },
  },
});
