import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    splitChunks: false,
    sideEffects: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
});
