import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    extensions: ['...', '.ts'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
});
