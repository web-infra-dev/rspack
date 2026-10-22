import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const resolve = (filename: string) =>
  path.resolve(import.meta.dirname, filename);

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        resource: /lib\.js/,
        use: [
          {
            loader: './loader-2.mjs',
          },
        ],
      },
      {
        resource: resolve('lib.js'),
        use: [
          {
            loader: './loader-1.mjs',
          },
        ],
      },
    ],
  },
});
