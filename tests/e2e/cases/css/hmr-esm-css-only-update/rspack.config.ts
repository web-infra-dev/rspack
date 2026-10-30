import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  output: {
    module: true,
    chunkFormat: 'module',
    chunkLoading: 'import',
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
      scriptLoading: 'module',
    }),
    new rspack.CssExtractRspackPlugin(),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
} satisfies Configuration;
