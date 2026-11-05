import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

// Should apply correctly even if the config is an array
// See https://github.com/web-infra-dev/rspack/issues/10745
export default defineConfig([
  {
    extends: [
      path.resolve(import.meta.dirname, 'base.rspack.config.mts'),
      path.resolve(import.meta.dirname, 'dev.rspack.config.mts'),
    ],
    entry: './src/index.js',
    output: {
      path: path.resolve(import.meta.dirname, 'dist'),
    },
  },
]);
