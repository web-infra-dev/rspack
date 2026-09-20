import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: ['./loader.mjs'],
      },
    ],
  },
});
