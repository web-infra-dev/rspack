import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index',
  stats: 'errors-warnings',
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
        },
      },
    ],
  },
});
