import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  resolve: {
    alias: {
      '@': path.join(import.meta.dirname, 'src'),
    },
  },
});
