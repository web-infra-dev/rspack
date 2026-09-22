import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
  },
});
