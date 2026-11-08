import { defineConfig } from '@rspack/cli';

export default defineConfig({
  experiments: {
    newCache: true,
  },
});
