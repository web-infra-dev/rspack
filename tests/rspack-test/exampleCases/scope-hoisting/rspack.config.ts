import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // mode: "development" || "production",
  optimization: {
    usedExports: true,
    concatenateModules: true,
    chunkIds: 'deterministic', // To keep filename consistent between different modes (for example building only)
  },
});
