import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  resolve: {
    alias: {
      alias_file: ['./file1', './file2'],
    },
  },
  cache: {
    type: 'persistent',
  },
});
