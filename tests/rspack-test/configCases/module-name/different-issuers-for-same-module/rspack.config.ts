import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: ['./a', './b', './test'],
  module: {
    rules: [
      {
        test: /c\.js/,
        issuer: /a\.js/,
        loader: './loader-a.mjs',
      },
      {
        test: /c\.js/,
        issuer: /b\.js/,
        loader: './loader-b.mjs',
      },
    ],
  },
});
