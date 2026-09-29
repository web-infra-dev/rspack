import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    chunkIds: 'deterministic',
  },
});
