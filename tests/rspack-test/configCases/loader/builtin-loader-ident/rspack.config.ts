import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
            },
            ident: 'builtin-swc-loader',
          },
        ],
      },
    ],
  },
});
