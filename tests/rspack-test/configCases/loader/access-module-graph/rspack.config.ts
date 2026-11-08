import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: './index.js',
  module: {
    rules: [
      {
        test: /index.js/,
        use: [{ loader: './access-mg-loader.mjs' }],
      },
    ],
  },
});
