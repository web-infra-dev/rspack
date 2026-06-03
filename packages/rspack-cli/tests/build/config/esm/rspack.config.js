import path from 'node:path';
import { fileURLToPath } from 'node:url';

export default {
  mode: 'production',
  entry: path.resolve(import.meta.dirname, 'main.ts'),
  output: {
    path: path.resolve(import.meta.dirname, 'dist'),
    filename: 'js.bundle.js',
  },
};
