import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js$/,
        use: [
          {
            loader: './loader.mjs',
            parallel: true,
            options: { nested: { transform: () => {} } },
          },
        ],
      },
    ],
  },
});
