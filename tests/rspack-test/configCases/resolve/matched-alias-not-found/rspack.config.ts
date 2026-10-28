import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    alias: {
      m1: path.resolve(import.meta.dirname, 'node_modules', 'm2', 'mod.js'),
    },
  },
});
