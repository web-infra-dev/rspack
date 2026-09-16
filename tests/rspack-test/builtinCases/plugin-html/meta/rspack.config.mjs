import { HtmlRspackPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
