import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  resolve: {
    mainFields: ['main'],
    preferRelative: true,
    tsConfig: path.join(import.meta.dirname, 'tsconfig.json'),
  },
});
