import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    modules: [
      path.resolve(import.meta.dirname, 'a'),
      path.resolve(import.meta.dirname, 'b'),
    ],
    alias: {
      [path.resolve(import.meta.dirname, 'a', 'foo')]: false,
    },
  },
});
