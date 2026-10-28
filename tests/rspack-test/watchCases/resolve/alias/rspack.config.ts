import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    alias: {
      'multi-alias': ['./b', './a'],
    },
  },
});
