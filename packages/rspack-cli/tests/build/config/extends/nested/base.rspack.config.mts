import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig({
  extends: path.resolve(import.meta.dirname, 'core.rspack.config.mts'),
  output: {
    filename: 'base.bundle.js',
  },
});
