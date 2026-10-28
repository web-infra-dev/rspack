import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                target: 'es2015',
              },
            },
          },
        ],
      },
    ],
  },
});
