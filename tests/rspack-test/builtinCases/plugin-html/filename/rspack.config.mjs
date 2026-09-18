import { HtmlRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new HtmlRspackPlugin({
      filename: '[name].[contenthash].html',
    }),
    new HtmlRspackPlugin({
      template: './index.html',
      filename: '[name].[contenthash].html',
    }),
  ],
};
