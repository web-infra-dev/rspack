import { defineConfig } from '@rspack/cli';

import HtmlRspackPlugin from 'html-rspack-plugin';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new HtmlRspackPlugin({
      template: './document.ejs',
    }),
    new rspack.DefinePlugin({
      title: JSON.stringify('CUSTOM TITLE'),
    }),
  ],
});
