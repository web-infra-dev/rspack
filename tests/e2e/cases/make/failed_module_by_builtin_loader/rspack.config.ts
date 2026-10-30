import { defineConfig } from '@rspack/cli';
import rspack from '@rspack/core';

export default defineConfig({
  entry: './src/index.js',
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: [/node_modules/],
        include: [/src/],
        loader: 'builtin:swc-loader',
      },
    ],
  },
  devServer: {
    hot: true,
  },
});
