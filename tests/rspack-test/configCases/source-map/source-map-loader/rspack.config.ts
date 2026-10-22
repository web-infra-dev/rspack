import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.[tj]sx?$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
        },
      },
      {
        test: /\.[tj]sx?$/,
        enforce: 'pre',
        loader: 'source-map-loader',
      },
    ],
  },
});
