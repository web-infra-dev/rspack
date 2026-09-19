import { defineConfig } from '@rspack/cli';

import { fileURLToPath } from 'node:url';

export default defineConfig({
  module: {
    rules: [
      {
        with: { type: 'RANDOM' },
        use: fileURLToPath(import.meta.resolve('./test-loader.mjs')),
      },
    ],
  },
});
