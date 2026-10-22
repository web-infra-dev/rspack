import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: './index.html',
      filename: 'index.html',
      minify: false,
    }),
    new rspack.HtmlRspackPlugin({
      template: './index.html',
      filename: 'index.minified.html',
      minify: true,
    }),
  ],
});
