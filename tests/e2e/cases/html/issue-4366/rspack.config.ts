import type { Configuration } from '@rspack/core';
import { rspack } from '@rspack/core';

export default {
  context: import.meta.dirname,
  entry: './src/index.js',
  stats: 'none',
  plugins: [new rspack.HtmlRspackPlugin({ template: './src/index.html' })],
} satisfies Configuration;
