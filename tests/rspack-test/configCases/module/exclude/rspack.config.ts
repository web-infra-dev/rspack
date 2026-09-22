import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const resolve = (filename: string) =>
  path.resolve(import.meta.dirname, filename);

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        exclude: /lib\.js/,
        use: [
          {
            loader: './loader.mjs',
          },
        ],
      },
      {
        exclude: resolve('index.js'),
        use: [
          {
            loader: './loader.mjs',
          },
        ],
      },
    ],
  },
});
