import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig([
  {
    extends: path.resolve(import.meta.dirname, 'base.rspack.config.mts'),
  },
  {
    name: 'derived_config2',
    mode: 'development',
    entry: './src/index2.js',
    output: {
      path: path.resolve(import.meta.dirname, 'dist'),
      filename: 'bundle2.js',
    },
  },
]);
