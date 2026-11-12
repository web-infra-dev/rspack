import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  module: {
    rules: [
      {
        test: /late\.js$/,
        use: path.resolve(import.meta.dirname, 'coalesce.loader.mjs'),
      },
    ],
  },
});
