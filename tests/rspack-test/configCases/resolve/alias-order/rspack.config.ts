import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  entry: './index.js',
  resolve: {
    alias: {
      '@foo/index': path.resolve(import.meta.dirname, './b'),
      '@foo': path.resolve(import.meta.dirname, './a'), // should not be used
    },
  },
});
