import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        resourceQuery: /^\?loader/,
        use: './loader.mjs?query',
      },
    ],
  },
});
