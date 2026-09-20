import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: [{ loader: './loader-b.mjs' }, { loader: './loader-a.mjs' }],
      },
    ],
  },
});
