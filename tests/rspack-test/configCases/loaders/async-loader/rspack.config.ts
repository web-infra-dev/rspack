import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: './loader-1.mjs',
      },
      {
        test: /b\.js$/,
        use: './loader-2.mjs',
      },
      {
        test: /c\.js$/,
        use: './loader-3.mjs',
      },
    ],
  },
});
