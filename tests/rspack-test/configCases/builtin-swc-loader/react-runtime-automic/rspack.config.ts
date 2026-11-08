import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.jsx',
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.jsx$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
          jsc: {
            transform: {
              react: {
                runtime: 'automatic',
                pragma: 'React.createElement',
                pragmaFrag: 'React.Fragment',
                throwIfNamespace: true,
                useBuiltins: false,
              },
            },
          },
        },
      },
    ],
  },
});
