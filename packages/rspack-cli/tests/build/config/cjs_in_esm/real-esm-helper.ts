import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dirname = path.dirname(fileURLToPath(import.meta.url));

export default {
  mode: 'production',
  entry: path.resolve(dirname, 'main.ts'),
  output: {
    path: path.resolve(dirname, 'dist'),
    filename: 'cts-real-esm-helper.bundle.js',
  },
};
