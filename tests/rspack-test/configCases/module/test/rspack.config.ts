import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const resolve = (filename: string) =>
  path.resolve(import.meta.dirname, filename);

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js/,
        use: [
          {
            loader: './loader-2.mjs',
          },
        ],
      },
      {
        test: resolve('lib.js'),
        use: [
          {
            loader: './loader-1.mjs',
          },
        ],
      },
      {
        test: /\.module\.less$/,
        type: 'css/module',
      },
      {
        test: /(?<!module).less$/,
        type: 'css',
      },
      {
        test: /\.svg$/i,
        type: 'asset',
      },
    ],
  },
});
