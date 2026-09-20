import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
