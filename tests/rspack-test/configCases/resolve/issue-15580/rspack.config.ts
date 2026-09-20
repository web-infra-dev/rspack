import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    modules: [
      'node_modules',
      path.resolve(import.meta.dirname, './node_modules'),
    ],
  },
});
