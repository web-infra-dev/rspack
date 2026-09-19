import { defineConfig } from '@rspack/cli';

export default defineConfig({
  incremental: {
    buildChunkGraph: true,
  },
});
