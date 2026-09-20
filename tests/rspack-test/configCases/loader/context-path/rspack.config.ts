import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'a.js'),
        use: [
          {
            loader: './my-loader.mjs',
          },
        ],
      },
    ],
  },
});
