import { defineConfig } from '@rspack/cli';

import path from 'node:path';
export default defineConfig({
  // mode: "development" || "production",
  entry: {
    main: ['./example.js'],
  },
  optimization: {
    splitChunks: {
      minSize: 0, // This example is too small, in practice you can use the defaults
    },
    chunkIds: 'deterministic', // To keep filename consistent between different modes (for example building only)
  },
  output: {
    path: path.resolve(import.meta.dirname, 'dist'),
    filename: 'output.js',
  },
});
