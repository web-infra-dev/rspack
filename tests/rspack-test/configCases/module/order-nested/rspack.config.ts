import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib.js/,
        rules: [
          {
            use: ['./loader2.mjs'],
          },
        ],
        oneOf: [
          {
            resourceQuery: /random-string/,
            use: ['./loader1.mjs'],
          },
          {
            use: ['./loader.mjs'],
          },
        ],
      },
    ],
  },
});
