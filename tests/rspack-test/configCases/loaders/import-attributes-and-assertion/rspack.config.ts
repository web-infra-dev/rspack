import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

// Rspack don't support assert since it's deprecated

export default defineConfig({
  module: {
    rules: [
      {
        with: { type: 'json' },
        loader: fileURLToPath(import.meta.resolve('./loader-with.mjs')),
      },
    ],
  },
});
