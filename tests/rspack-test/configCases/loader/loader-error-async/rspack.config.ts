import { defineConfig } from '@rspack/cli';

import path from 'node:path';

const file = path.resolve(import.meta.dirname, 'lib.js');

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: file,
        resourceQuery: /async/,
        use: [
          {
            loader: './async.mjs',
          },
        ],
      },
      {
        test: file,
        resourceQuery: /callback/,
        use: [
          {
            loader: './callback.mjs',
          },
        ],
      },
    ],
  },
});
