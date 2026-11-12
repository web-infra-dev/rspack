import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: './loader.mjs',
      },
    ],
  },
});
