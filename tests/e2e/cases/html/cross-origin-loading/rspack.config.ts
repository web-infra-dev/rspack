import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  mode: 'development',
  entry: './src/index.js',
  stats: 'none',
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
    }),
  ],
  output: {
    crossOriginLoading: 'anonymous',
  },
  devServer: {
    port: 3000,
  },
  // for concise assert for assets
  lazyCompilation: false,
} satisfies Configuration;
