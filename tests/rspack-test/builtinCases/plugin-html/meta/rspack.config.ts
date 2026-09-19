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
      meta: {
        viewport: {
          name: 'viewport',
          content: 'width=device-width, initial-scale=1, shrink-to-fit=no',
        },
      },
    }),
  ],
});
