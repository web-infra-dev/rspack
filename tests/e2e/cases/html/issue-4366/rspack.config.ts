import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  entry: './src/index.js',
  stats: 'none',
  plugins: [new rspack.HtmlRspackPlugin({ template: './src/index.html' })],
});
