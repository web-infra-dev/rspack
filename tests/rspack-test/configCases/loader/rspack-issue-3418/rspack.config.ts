import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
});
