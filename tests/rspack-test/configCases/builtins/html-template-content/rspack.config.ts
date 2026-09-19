import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      templateContent:
        '<!DOCTYPE html><html><body><div><%= env %></div></body></html>',
      templateParameters: {
        env: 'production',
      },
    }),
  ],
});
