import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig([
  {
    name: 'derived_config1',
    extends: path.resolve(import.meta.dirname, 'base.rspack.config.mts'),
    entry: './src/index1.js',
    output: {
      path: path.resolve(import.meta.dirname, 'dist'),
      filename: 'bundle1.js',
    },
  },
  {
    name: 'derived_config2',
    extends: path.resolve(import.meta.dirname, 'base.rspack.config.mts'),
    entry: './src/index2.js',
    output: {
      path: path.resolve(import.meta.dirname, 'dist'),
      filename: 'bundle2.js',
    },
  },
]);
