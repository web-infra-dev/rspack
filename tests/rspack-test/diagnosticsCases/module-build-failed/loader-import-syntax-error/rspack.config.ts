import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index\.js/,
        loader: './loader.mjs',
        options: {},
      },
    ],
  },
});
