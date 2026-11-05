import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig({
  extends: path.resolve(import.meta.dirname, 'non-existent-config.js'),
  entry: './src/index.js',
});
