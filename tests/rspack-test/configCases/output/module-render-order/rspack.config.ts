import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: path.join(import.meta.dirname, 'src'),
  output: {
    filename: 'main.js',
    chunkFilename: '[name].js',
  },
  optimization: {
    minimize: false,
    concatenateModules: false,
    moduleIds: 'deterministic',
  },
});
