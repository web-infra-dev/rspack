import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'nosources-source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  resolve: {
    extensions: ['...', '.ts'],
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
  plugins: [
    new rspack.DefinePlugin({
      CONTEXT: JSON.stringify(import.meta.dirname),
    }),
  ],
});
