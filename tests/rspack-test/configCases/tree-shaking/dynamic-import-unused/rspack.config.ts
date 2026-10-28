import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  output: {
    chunkFilename: 'chunk.js',
  },
  optimization: {
    minimize: true,
    providedExports: true,
    usedExports: true,
    sideEffects: true,
    innerGraph: true,
  },
});
