import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    index: {
      import: ['./index.js'],
    },
  },
  optimization: {
    providedExports: true,
    usedExports: 'global',
  },
});
