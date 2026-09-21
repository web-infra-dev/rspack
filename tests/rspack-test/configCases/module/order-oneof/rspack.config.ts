import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['./loader.mjs'],
        oneOf: [
          {
            test: /lib\.js$/,
            use: ['./loader1.mjs'],
          },
          {
            test: /random-string/,
            use: ['./loader2.mjs'],
          },
        ],
      },
    ],
  },
});
