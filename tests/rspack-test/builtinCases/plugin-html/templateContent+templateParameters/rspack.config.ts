import { defineConfig } from '@rspack/cli';

import { HtmlRspackPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    index: {
      import: ['./index.js'],
    },
  },
  plugins: [
    new HtmlRspackPlugin({
      templateContent:
        '<!DOCTYPE html><html><body><div><%= foo %></div></body></html>',
      templateParameters: {
        foo: 'bar',
      },
    }),
  ],
});
