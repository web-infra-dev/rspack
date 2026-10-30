import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: './src/index.js',
  mode: 'development',
  stats: 'none',
  devtool: false,
  optimization: {
    usedExports: false,
    sideEffects: 'flag',
  },
  plugins: [new rspack.HtmlRspackPlugin()],
  lazyCompilation: true,
  devServer: { hot: true },
} satisfies Configuration;
