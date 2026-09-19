import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  target: ['web', 'es2017'],
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
