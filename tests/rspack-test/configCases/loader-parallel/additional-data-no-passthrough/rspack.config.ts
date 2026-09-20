import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'a.js'),
        use: [
          { loader: './loader-2.mjs', parallel: true, options: {} },
          {
            loader: 'builtin:test-no-passthrough-loader',
            parallel: true,
            options: {},
          },
          { loader: './loader-1.mjs', parallel: true, options: {} },
        ],
      },
    ],
  },
});
