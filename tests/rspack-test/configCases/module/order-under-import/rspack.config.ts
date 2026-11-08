import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /index\.js/,
        use: [
          {
            loader: './test-loader.mjs',
          },
        ],
      },
    ],
  },
});
