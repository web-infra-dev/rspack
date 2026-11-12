import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    chunkIds: 'deterministic', // To keep filename consistent between different modes (for example building only)
  },
});
