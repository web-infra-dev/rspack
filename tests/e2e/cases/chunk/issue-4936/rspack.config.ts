import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: './src/index.js',
  stats: 'none',
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
} satisfies Configuration;
