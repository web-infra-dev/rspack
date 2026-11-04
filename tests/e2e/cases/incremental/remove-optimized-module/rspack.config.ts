import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  entry: './index.js',
  cache: true,
  plugins: [new rspack.HtmlRspackPlugin()],
});
