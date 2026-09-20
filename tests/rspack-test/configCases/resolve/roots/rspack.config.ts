import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  resolve: {
    roots: [import.meta.dirname],
  },
});
