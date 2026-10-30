import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig({
  extends: 'mock-rspack-config',
  entry: './src/index.js',
  output: {
    path: path.resolve(import.meta.dirname, 'dist'),
  },
});
