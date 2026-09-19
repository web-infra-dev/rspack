import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  devtool: false,
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
        },
      },
    ],
  },
  plugins: [
    new rspack.DefinePlugin({
      CONTEXT: JSON.stringify(import.meta.dirname),
    }),
    new rspack.SourceMapDevToolPlugin({
      filename: '[file].map[query]',
    }),
  ],
});
