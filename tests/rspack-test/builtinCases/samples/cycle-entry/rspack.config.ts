import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    index: {
      import: ['./index.js'],
    },
    index2: {
      import: ['./index2.js'],
    },
  },
  optimization: {
    removeAvailableModules: true,
    providedExports: true,
    usedExports: 'global',
  },
});
