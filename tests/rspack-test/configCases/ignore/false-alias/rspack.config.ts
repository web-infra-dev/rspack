import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './test.js',
  resolve: {
    alias: {
      'ignored-module': false,
      './ignored-module': false,
    },
  },
});
