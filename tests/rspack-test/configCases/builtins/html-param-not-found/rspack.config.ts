import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      templateContent:
        '<!DOCTYPE html><html><head><title><%= title %></title></head><body></body></html>',
    }),
  ],
});
