import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './a.js',
    b: './b.js',
  },
  module: {
    rules: [
      {
        test: /a\.js/,
        use: [
          {
            loader: 'builtin:swc-loader',
          },
        ],
      },
      {
        test: /b\.js/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                output: {
                  charset: 'ascii',
                },
              },
            },
          },
        ],
      },
    ],
  },
});
