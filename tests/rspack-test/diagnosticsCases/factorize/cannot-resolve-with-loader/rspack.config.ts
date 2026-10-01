import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
});
