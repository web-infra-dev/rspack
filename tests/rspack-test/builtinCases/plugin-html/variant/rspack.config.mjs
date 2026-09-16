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
      filename: 'output.html',
      template: 'input.html',
      inject: 'head',
      scriptLoading: 'blocking',
    }),
  ],
};
