import HtmlRspackPlugin from 'html-rspack-plugin';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new HtmlRspackPlugin({
      template: './document.ejs',
    }),
    new rspack.DefinePlugin({
      title: JSON.stringify('CUSTOM TITLE'),
    }),
  ],
};
