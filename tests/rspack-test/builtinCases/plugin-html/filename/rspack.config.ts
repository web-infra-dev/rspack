import { defineConfig } from '@rspack/cli';

import { HtmlRspackPlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new HtmlRspackPlugin({
      filename: '[name].[contenthash].html',
    }),
    new HtmlRspackPlugin({
      template: './index.html',
      filename: '[name].[contenthash].html',
    }),
  ],
});
