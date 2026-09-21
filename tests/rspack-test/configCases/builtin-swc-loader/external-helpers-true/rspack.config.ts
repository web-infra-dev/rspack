import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    extensions: ['...', '.ts'],
  },
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                externalHelpers: true,
                target: 'es5',
              },
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
});
