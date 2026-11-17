import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: ['./loader.cjs'],
      },
    ],
  },
});
