import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
      },
    ],
  },
} satisfies Configuration;
