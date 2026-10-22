import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  // mode: "development" || "production",
  entry: {
    main: './example',
  },
  optimization: {
    runtimeChunk: true,
  },
  output: {
    path: path.join(import.meta.dirname, 'dist'),
    filename: '[name].chunkhash.js',
    chunkFilename: '[name].chunkhash.js',
  },
});
