import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  resolve: {
    alias: {
      'image.png': false,
    },
  },
});
