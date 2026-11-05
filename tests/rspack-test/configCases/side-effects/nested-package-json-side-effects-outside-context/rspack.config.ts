import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  context: path.join(import.meta.dirname, 'src'),
  mode: 'production',
  entry: './index.js',
  resolve: {
    modules: [path.join(import.meta.dirname, 'node_modules'), 'node_modules'],
  },
  optimization: {
    sideEffects: true,
    usedExports: true,
    providedExports: true,
    concatenateModules: false,
    minimize: false,
  },
});
