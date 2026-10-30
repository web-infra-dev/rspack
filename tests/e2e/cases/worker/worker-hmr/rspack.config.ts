import type { Configuration } from '@rspack/core';
import rspack from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: {
    main: './src/index.js',
  },
  devtool: false,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin({ template: './src/index.html' })],
  devServer: {
    hot: true,
  },
} satisfies Configuration;
