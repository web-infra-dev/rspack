import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig({
  entry: './src/index1.js',
  name: 'base_config',
  mode: 'development',
  output: {
    path: path.resolve(import.meta.dirname, 'dist'),
    filename: 'bundle1.js',
  },
});
