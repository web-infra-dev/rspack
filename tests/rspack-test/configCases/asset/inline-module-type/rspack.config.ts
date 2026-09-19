import { defineConfig } from '@rspack/cli';

import { fileURLToPath } from 'node:url';
export default defineConfig({
  module: {
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/source',
        resolve: {
          alias: { value: fileURLToPath(import.meta.resolve('./value.js')) },
        },
      },
    ],
  },
});
