import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  entry: './src/index.js',
  stats: 'none',
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
};
