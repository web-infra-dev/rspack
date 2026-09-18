import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.HtmlRspackPlugin({
      templateContent:
        '<!DOCTYPE html><html><head><title><%= title %></title></head><body></body></html>',
    }),
  ],
};
