import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js/,
        use: [
          {
            loader: './unclonable.mjs',
            options: {
              notclonable() {},
            },
          },
          {
            loader: './loader-in-worker.mjs',
            parallel: true,
            options: {},
          },
        ],
      },
    ],
  },
});
