import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    alias: {
      '#': path.resolve(import.meta.dirname, '#'),
    },
    fallback: {
      './b': path.resolve(import.meta.dirname, 'a'),
    },
  },
});
