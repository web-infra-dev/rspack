import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    index: {
      import: ['./index.js'],
    },
  },
  optimization: {
    removeAvailableModules: true,
    providedExports: true,
    usedExports: 'global',
  },
});
