import { defineConfig } from '../../../../../dist/index.js';
import path from 'node:path';

export default defineConfig(() => {
  return {
    extends: [import.meta.resolve('./base.rspack.config.json')],
    entry: './src/index.js',
    output: {
      path: path.resolve(import.meta.dirname, 'dist'),
    },
  };
});
