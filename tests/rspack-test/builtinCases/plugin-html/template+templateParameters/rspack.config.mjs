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
      template: 'index.html',
      templateParameters: {
        foo: 'bar',
      },
    }),
  ],
};
