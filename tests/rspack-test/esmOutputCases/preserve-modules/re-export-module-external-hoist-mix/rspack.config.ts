import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  entry: './src/index.js',
  externals: {
    fs: 'fs',
  },
  output: {
    library: {
      type: 'modern-module',
      preserveModules: path.resolve(import.meta.dirname, 'src'),
    },
  },
});
