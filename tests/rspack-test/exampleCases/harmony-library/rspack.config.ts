import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  // mode: "development" || "production",
  entry: './example',
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: 'MyLibrary.umd.js',
    library: { type: 'umd', name: 'MyLibrary' },
  },
});
