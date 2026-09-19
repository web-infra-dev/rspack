import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  resolve: {
    alias: {
      _: [
        path.resolve(import.meta.dirname, 'a'),
        path.resolve(import.meta.dirname, 'b'),
      ],
    },
  },
});
