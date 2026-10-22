import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  module: {
    rules: [
      {
        test: /.txt$/,
        loader: path.resolve(import.meta.dirname, './loader.mjs'),
      },
    ],
  },
  resolve: {
    extensions: ['...', '.txt'],
  },
});
