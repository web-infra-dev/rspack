import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    splitChunks: false,
  },
  incremental: {
    buildChunkGraph: true,
  },
});
