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
      template: 'index.html',
      templateParameters: {
        foo: 'bar',
      },
    }),
  ],
});
