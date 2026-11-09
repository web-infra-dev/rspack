import path from 'node:path';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: path.join(import.meta.dirname, 'project'),
  entry: './index.js',
});
