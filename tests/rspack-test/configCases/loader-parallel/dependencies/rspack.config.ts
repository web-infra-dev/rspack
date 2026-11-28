import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /index\.js$/,
        use: [
          {
            loader: './loader.mjs',
            parallel: true,
            options: {},
          },
        ],
      },
    ],
  },
});
