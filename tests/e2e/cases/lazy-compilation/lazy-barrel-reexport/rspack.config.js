import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
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
};
