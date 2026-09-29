import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: './loader.mjs',
      },
    ],
  },
});
