import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    packageMap: path.resolve(import.meta.dirname, 'package-map.json'),
    alias: {
      aliased: 'dep',
    },
    fallback: {
      missing: path.resolve(import.meta.dirname, 'fallback.js'),
    },
    aliasFields: ['browser'],
  },
});
