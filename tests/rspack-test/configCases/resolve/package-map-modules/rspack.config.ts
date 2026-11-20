import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  resolve: {
    packageMap: path.resolve(import.meta.dirname, 'package-map.json'),
    modules: [path.resolve(import.meta.dirname, 'modules'), 'node_modules'],
  },
});
