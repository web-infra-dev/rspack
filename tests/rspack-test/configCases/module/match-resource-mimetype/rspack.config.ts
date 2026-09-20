import { defineConfig } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  module: {
    rules: [
      {
        include: path.resolve(import.meta.dirname, 'a.js'),
        use: [
          './get-source.mjs',
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                target: 'es3',
              },
            },
          },
        ],
      },
    ],
  },
});
