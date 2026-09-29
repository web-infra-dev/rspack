import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index\.js/,
        use: ['./import-loader.mjs', './import-loader-2.mjs'],
      },
    ],
  },
});
