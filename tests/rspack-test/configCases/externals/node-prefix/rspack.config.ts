import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  target: 'node',
  resolve: {
    alias: {
      './node:local': path.resolve(import.meta.dirname, 'local.js'),
    },
  },
});
