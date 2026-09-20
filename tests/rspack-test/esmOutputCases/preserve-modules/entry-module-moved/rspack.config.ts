import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  entry: {
    index: './src/index.js',
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
});
