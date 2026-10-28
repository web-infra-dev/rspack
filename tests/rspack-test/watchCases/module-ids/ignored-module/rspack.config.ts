import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    alias: {
      ignored: false,
    },
  },
});
