import { defineConfig } from '@rspack/cli';
import { HtmlRspackPlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new HtmlRspackPlugin({
      filename: 'inject_head.html',
      inject: 'head',
    }),
    new HtmlRspackPlugin({
      filename: 'inject_body.html',
      inject: 'body',
    }),
    new HtmlRspackPlugin({
      filename: 'inject_false.html',
      inject: false,
    }),
  ],
});
