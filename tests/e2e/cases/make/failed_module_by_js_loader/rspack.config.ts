import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
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
        loader: './loader.mjs',
      },
    ],
  },
  devServer: {
    hot: true,
  },
} satisfies Configuration;
