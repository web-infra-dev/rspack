import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    removeAvailableModules: true,
    providedExports: true,
    usedExports: 'global',
  },
});
