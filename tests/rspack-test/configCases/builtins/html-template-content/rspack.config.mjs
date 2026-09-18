import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.HtmlRspackPlugin({
      templateContent:
        '<!DOCTYPE html><html><body><div><%= env %></div></body></html>',
      templateParameters: {
        env: 'production',
      },
    }),
  ],
};
