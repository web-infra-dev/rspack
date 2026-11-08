import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
        },
      },
    ],
  },
});
