import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'persistent', portable: true },
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: ['./loader.cjs'],
      },
    ],
  },
});
