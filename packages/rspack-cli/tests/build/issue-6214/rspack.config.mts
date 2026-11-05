import { defineConfig } from '../../../dist/index.js';
import path from 'node:path';

export default defineConfig({
  mode: 'development', // will be override to "production" by "--mode"
  entry: './entry.js',
  output: {
    clean: true,
    path: path.resolve(import.meta.dirname, 'dist'),
  },
  optimization: {
    nodeEnv: false,
  },
});
