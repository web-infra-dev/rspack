import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  mode: 'development',
  entry: './index.js',
  cache: true,
  plugins: [new rspack.HtmlRspackPlugin()],
} satisfies Configuration;
