import type { Configuration } from '@rspack/core';
import rspack from '@rspack/core';

export default {
  entry: './src/index.js',
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  optimization: {
    emitOnErrors: false,
  },
  devServer: {
    hot: true,
  },
} satisfies Configuration;
