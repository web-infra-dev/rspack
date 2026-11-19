import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: 'index.html',
      filename: 'page.html',
    }),
    new rspack.HtmlRspackPlugin({
      template: 'index.html',
      templateContent: '<!DOCTYPE html><html><body></extra></body></html>',
      filename: 'inline.html',
    }),
    new rspack.HtmlRspackPlugin({ filename: 'default-page.html' }),
  ],
});
