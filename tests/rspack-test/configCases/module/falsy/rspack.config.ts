import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      undefined,
      {
        test: /lib.js/,
        use: ['./loader2.mjs'],
      },
      {
        test: /lib.js/,
        oneOf: [
          undefined,
          {
            resourceQuery: '/(__inline=false|url)/',
            use: ['./loader1.mjs'],
          },
          {
            use: ['./loader.mjs'],
          },
          {
            use: ['./loader1.mjs'],
          },
        ],
      },
    ],
  },
});
