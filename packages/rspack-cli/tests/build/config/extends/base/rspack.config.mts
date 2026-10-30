import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig({
  extends: path.resolve(import.meta.dirname, 'base.rspack.config.mts'),
  entry: './src/index.js',
  output: {
    path: path.resolve(import.meta.dirname, 'dist'),
  },
});
