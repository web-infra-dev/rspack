import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
  },
  resolve: {
    packageMap: path.join(import.meta.dirname, 'package-map.json'),
  },
});
