import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const resolve = (filename: string) =>
  path.resolve(import.meta.dirname, filename);

export default defineConfig({
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: resolve('index.js'),
        use: [
          {
            loader: './test-loader.mjs',
          },
        ],
      },
    ],
  },
});
