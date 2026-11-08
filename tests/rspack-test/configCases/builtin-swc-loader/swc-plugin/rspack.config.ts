import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.js$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            detectSyntax: 'auto',
            jsc: {
              experimental: {
                plugins: [
                  [
                    '@swc/plugin-remove-console',
                    {
                      exclude: ['error'],
                    },
                  ],
                ],
              },
            },
          },
        },
        type: 'javascript/auto',
      },
    ],
  },
});
