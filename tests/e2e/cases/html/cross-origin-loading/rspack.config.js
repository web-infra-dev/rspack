import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
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
};
